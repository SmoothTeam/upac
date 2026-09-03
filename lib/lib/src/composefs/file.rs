// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{File, Metadata, read_dir, read_link};
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use composefs::MAX_INLINE_CONTENT;
use composefs::generic_tree::Stat;
use composefs::repository::{ImportContext, Repository};
use composefs::tree::{Directory, FileSystem, Inode, LeafContent, RegularFile};

use upac_abi::hook::CancelToken;

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

    pub fn ensure_parents_in_tree(&self, tree: &mut FileSystem<ObjectID>) -> Result<(), RepoError> {
        let Some(parent) = self.path.parent() else {
            return Ok(());
        };

        let mut current = PathBuf::new();

        for component in parent.components() {
            current.push(component);

            let ancestor = FileHandle::new(current.clone());
            if ancestor.stat_in_tree(tree).is_err() {
                ancestor.insert_in_tree(tree, Stat::uninitialized())?;
            }
        }

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

    pub fn copy_from_tree(
        &self, dest_tree: &mut FileSystem<ObjectID>, source_tree: &FileSystem<ObjectID>, source_path: &Path,
    ) -> Result<(), RepoError> {
        let (source_parent, source_filename) = source_tree.root.split(source_path.as_os_str())?;
        let leaf_id = source_parent.leaf_id(source_filename)?;
        let leaf = source_tree.leaf(leaf_id).clone();

        let new_leaf_id = dest_tree.push_leaf(leaf.stat, leaf.content);

        let (parent, filename) = dest_tree.root.split_mut(self.path.as_os_str())?;
        parent.insert(filename, Inode::leaf(new_leaf_id));

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
            RegularFile::External(object_id, _size) | RegularFile::ExternalNoVerity(object_id, _size) => {
                Ok(repository.read_object(object_id)?)
            }
            RegularFile::Sparse(size) => Ok(vec![0u8; *size as usize]),
        }
    }

    pub fn import_directory(
        &self, repository: &Repository<ObjectID>, tree: &mut FileSystem<ObjectID>, source_dir: &Path,
        ctx: &mut ImportContext, cancel: &CancelToken, on_entry: &mut dyn FnMut(&Path),
    ) -> Result<Vec<PathBuf>, RepoError> {
        let mut imported = Vec::new();

        for entry in read_dir(source_dir)? {
            if cancel.is_cancelled() {
                return Err(RepoError::Cancelled);
            }

            let entry = entry?;
            let source_path = entry.path();
            let metadata = entry.metadata()?;
            let stat = stat_from_metadata(&metadata);
            let name = PathBuf::from(entry.file_name());
            let target = FileHandle::new(self.path.join(&name));

            if metadata.is_dir() {
                target.insert_in_tree(tree, stat)?;
                let nested = target.import_directory(repository, tree, &source_path, ctx, cancel, on_entry)?;
                imported.extend(nested.into_iter().map(|relative| name.join(relative)));
            } else if metadata.is_symlink() {
                target.symlink_in_tree(tree, read_link(&source_path)?, stat)?;
                on_entry(&name);
                imported.push(name);
            } else {
                target.insert_file(repository, tree, &File::open(&source_path)?, stat, ctx)?;
                on_entry(&name);
                imported.push(name);
            }
        }

        Ok(imported)
    }
}

pub(crate) fn stat_from_metadata(metadata: &Metadata) -> Stat {
    Stat {
        st_mode: metadata.mode(),
        st_uid: metadata.uid(),
        st_gid: metadata.gid(),
        st_mtim_sec: metadata.mtime(),
        st_mtim_nsec: metadata.mtime_nsec() as u32,
        xattrs: BTreeMap::new(),
    }
}
