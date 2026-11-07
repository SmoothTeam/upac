// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::BTreeMap;
use std::path::Path;

use composefs::generic_tree::{LeafContent, Stat};
use composefs::tree::{Directory, FileSystem, Inode, RegularFile};

use super::Tree;
use super::view::TreeView;

use crate::ObjectID;
use crate::error::RepoError;
use crate::layout::tree::DIR_MODE;

const DIRECTORY_STAT: Stat = Stat {
    st_mode: DIR_MODE,
    st_uid: 0,
    st_gid: 0,
    st_mtim_sec: 0,
    st_mtim_nsec: 0,
    xattrs: BTreeMap::new(),
};

pub struct TreeBuilder {
    pub(super) filesystem: FileSystem<ObjectID>,
}

impl TreeBuilder {
    pub fn new() -> Self {
        Self {
            filesystem: FileSystem::new(DIRECTORY_STAT),
        }
    }

    pub fn insert(
        &mut self, path: impl AsRef<Path>, content: LeafContent<RegularFile<ObjectID>>, stat: Stat,
    ) -> Result<(), RepoError> {
        let leaf_id = self.filesystem.push_leaf(stat, content);

        let (parent, filename) = self.filesystem.root.split_mut(path.as_ref().as_os_str())?;
        parent.insert(filename, Inode::leaf(leaf_id));

        Ok(())
    }

    pub fn insert_dir(&mut self, path: impl AsRef<Path>, stat: Stat) -> Result<(), RepoError> {
        let (parent, filename) = self.filesystem.root.split_mut(path.as_ref().as_os_str())?;
        parent.insert(filename, Inode::Directory(Box::new(Directory::new(stat))));

        Ok(())
    }

    pub fn insert_hard_link(&mut self, path: impl AsRef<Path>, original: impl AsRef<Path>) -> Result<(), RepoError> {
        let (original_parent, original_filename) = self.filesystem.root.split(original.as_ref().as_os_str())?;
        let leaf_id = original_parent.leaf_id(original_filename)?;

        let (parent, filename) = self.filesystem.root.split_mut(path.as_ref().as_os_str())?;
        parent.insert(filename, Inode::leaf(leaf_id));

        Ok(())
    }

    pub fn remove(&mut self, path: impl AsRef<Path>) -> Result<(), RepoError> {
        let (parent, filename) = self.filesystem.root.split_mut(path.as_ref().as_os_str())?;
        parent.remove(filename);

        Ok(())
    }

    pub fn update(
        &mut self, path: impl AsRef<Path>, content: LeafContent<RegularFile<ObjectID>>,
    ) -> Result<(), RepoError> {
        let (parent, filename) = self.filesystem.root.split(path.as_ref().as_os_str())?;
        let leaf_id = parent.leaf_id(filename)?;

        self.filesystem.leaf_mut(leaf_id).content = content;

        Ok(())
    }

    pub fn update_metadata(&mut self, path: impl AsRef<Path>, stat: Stat) -> Result<(), RepoError> {
        let path = path.as_ref();

        if let Ok(directory) = self.filesystem.root.get_directory_mut(path.as_os_str()) {
            directory.stat = stat;

            return Ok(());
        }

        let (parent, filename) = self.filesystem.root.split(path.as_os_str())?;
        let leaf_id = parent.leaf_id(filename)?;

        self.filesystem.leaf_mut(leaf_id).stat = stat;

        Ok(())
    }

    pub fn view(&self) -> TreeView<'_> {
        TreeView::new(&self.filesystem)
    }

    pub fn commit(self) -> Tree {
        Tree::from(self.filesystem)
    }
}

impl Default for TreeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<FileSystem<ObjectID>> for TreeBuilder {
    fn from(filesystem: FileSystem<ObjectID>) -> Self {
        Self { filesystem }
    }
}

impl From<Tree> for TreeBuilder {
    fn from(tree: Tree) -> Self {
        Self {
            filesystem: tree.filesystem,
        }
    }
}
