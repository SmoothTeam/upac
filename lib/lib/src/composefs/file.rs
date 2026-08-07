// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use composefs::MAX_INLINE_CONTENT;
use composefs::generic_tree::Stat;
use composefs::repository::{ImportContext, Repository};
use composefs::tree::{Directory, FileSystem, Inode, LeafContent, RegularFile};

use crate::composefs::error::RepoError;
use crate::composefs::repository::ObjectID;

pub struct FileHandle {
    path: PathBuf,
}

impl FileHandle {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn from_tree(tree: &FileSystem<ObjectID>, path: impl Into<PathBuf>) -> Result<Self, RepoError> {
        let path = path.into();

        let (parent, filename) = tree.root.split(path.as_os_str())?;
        parent.lookup(filename).ok_or(RepoError::NotFound)?;

        Ok(Self { path })
    }
}

impl FileHandle {
    pub fn insert_in_tree(&self, tree: &mut FileSystem<ObjectID>, stat: Stat) -> Result<(), RepoError> {
        let (parent, filename) = tree.root.split_mut(self.path.as_os_str())?;
        parent.insert(filename, Inode::Directory(Box::new(Directory::new(stat))));

        Ok(())
    }

    pub fn update_in_tree(&self, tree: &mut FileSystem<ObjectID>, stat: Stat) -> Result<(), RepoError> {
        tree.root.get_directory_mut(self.path.as_os_str())?.stat = stat;

        Ok(())
    }

    pub fn symlink_in_tree(
        &self, tree: &mut FileSystem<ObjectID>, target: impl AsRef<OsStr>, stat: Stat,
    ) -> Result<(), RepoError> {
        let leaf_id = tree.push_leaf(stat, LeafContent::Symlink(target.as_ref().into()));

        let (parent, filename) = tree.root.split_mut(self.path.as_os_str())?;
        parent.insert(filename, Inode::leaf(leaf_id));

        Ok(())
    }

    pub fn hardlink_in_tree(&self, tree: &mut FileSystem<ObjectID>, source: &Path) -> Result<(), RepoError> {
        let (source_parent, source_filename) = tree.root.split(source.as_os_str())?;
        let leaf_id = source_parent.leaf_id(source_filename)?;

        let (parent, filename) = tree.root.split_mut(self.path.as_os_str())?;
        parent.insert(filename, Inode::leaf(leaf_id));

        Ok(())
    }

    pub fn rename_in_tree(
        &mut self, tree: &mut FileSystem<ObjectID>, new_path: impl Into<PathBuf>,
    ) -> Result<(), RepoError> {
        let new_path = new_path.into();

        let (old_parent, old_filename) = tree.root.split_mut(self.path.as_os_str())?;
        let inode = old_parent.pop(old_filename).ok_or(RepoError::NotFound)?;

        let (new_parent, new_filename) = tree.root.split_mut(new_path.as_os_str())?;
        new_parent.insert(new_filename, inode);

        self.path = new_path;

        Ok(())
    }

    pub fn remove_in_tree(&self, tree: &mut FileSystem<ObjectID>) -> Result<(), RepoError> {
        let (parent, filename) = tree.root.split_mut(self.path.as_os_str())?;
        parent.remove(filename);

        Ok(())
    }
}

impl FileHandle {
    pub fn stat_in_tree<'t>(&self, tree: &'t FileSystem<ObjectID>) -> Result<&'t Stat, RepoError> {
        let (parent, filename) = tree.root.split(self.path.as_os_str())?;
        let inode = parent.lookup(filename).ok_or(RepoError::NotFound)?;

        Ok(inode.stat(&tree.leaves))
    }

    pub fn symlink_target_in_tree<'t>(&self, tree: &'t FileSystem<ObjectID>) -> Result<&'t OsStr, RepoError> {
        let (parent, filename) = tree.root.split(self.path.as_os_str())?;
        let inode = parent.lookup(filename).ok_or(RepoError::NotFound)?;

        let Inode::Leaf(leaf_id, _) = inode else {
            return Err(RepoError::IsADirectory);
        };

        match &tree.leaf(*leaf_id).content {
            LeafContent::Symlink(target) => Ok(target),
            _ => Err(RepoError::NotASymlink),
        }
    }

    pub fn list_in_tree<'t>(
        &self, tree: &'t FileSystem<ObjectID>,
    ) -> Result<impl Iterator<Item = (&'t OsStr, &'t Inode<ObjectID>)>, RepoError> {
        Ok(tree.root.get_directory(self.path.as_os_str())?.sorted_entries())
    }
}

impl FileHandle {
    pub fn insert_file(
        &self, repository: &Repository<ObjectID>, tree: &mut FileSystem<ObjectID>, source: &File, stat: Stat,
        ctx: &mut ImportContext,
    ) -> Result<(), RepoError> {
        let size = source.metadata()?.len();

        let regular = if size <= MAX_INLINE_CONTENT as u64 {
            let mut content = Vec::with_capacity(size as usize);
            let mut reader = source;
            reader.read_to_end(&mut content)?;

            RegularFile::Inline(content.into())
        } else {
            let (object_id, _method) = repository.ensure_object_from_file(source, size, ctx)?;

            RegularFile::External(object_id, size)
        };

        let leaf_id = tree.push_leaf(stat, LeafContent::Regular(regular));

        let (parent, filename) = tree.root.split_mut(self.path.as_os_str())?;
        parent.insert(filename, Inode::leaf(leaf_id));

        Ok(())
    }

    pub fn replace_file(
        &self, repository: &Repository<ObjectID>, tree: &mut FileSystem<ObjectID>, source: &File, stat: Stat,
        ctx: &mut ImportContext,
    ) -> Result<(), RepoError> {
        self.insert_file(repository, tree, source, stat, ctx)
    }

    pub fn read_file(
        &self, repository: &Repository<ObjectID>, tree: &FileSystem<ObjectID>,
    ) -> Result<Vec<u8>, RepoError> {
        let (parent, filename) = tree.root.split(self.path.as_os_str())?;
        let regular = parent.get_file(filename, &tree.leaves)?;

        match regular {
            RegularFile::Inline(content) => Ok(content.to_vec()),
            RegularFile::External(object_id, _size) => Ok(repository.read_object(object_id)?),
        }
    }
}
