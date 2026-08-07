// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use composefs::MAX_INLINE_CONTENT;
use composefs::generic_tree::{Inode, Stat};
use composefs::repository::{ImportContext, Repository};
use composefs::tree::{Directory, FileSystem, LeafContent, RegularFile};

use crate::composefs::error::RepoError;
use crate::composefs::repository::ObjectID;

pub struct FileHandle {
    path: PathBuf,
}

impl FileHandle {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
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
}
