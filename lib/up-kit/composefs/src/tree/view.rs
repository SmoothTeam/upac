// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use composefs::dumpfile::write_dumpfile;
use composefs::generic_tree::{LeafContent, Stat};
use composefs::tree::{Directory, FileSystem, Inode, RegularFile};

use upac_types::package::ObjectID;

use super::Tree;

use crate::error::RepoError;

#[derive(Clone, Copy)]
pub struct TreeView<'tree> {
    filesystem: &'tree FileSystem<ObjectID>,
}

pub struct TreeWalk<'tree> {
    pending: Vec<(PathBuf, &'tree Inode<ObjectID>)>,
}

impl<'tree> TreeView<'tree> {
    pub fn new(filesystem: &'tree FileSystem<ObjectID>) -> Self {
        Self { filesystem }
    }

    pub fn filesystem(&self) -> &'tree FileSystem<ObjectID> {
        self.filesystem
    }

    pub fn get(&self, path: impl AsRef<Path>) -> Result<LeafContent<RegularFile<ObjectID>>, RepoError> {
        let (parent, filename) = self.filesystem.root.split(path.as_ref().as_os_str())?;

        Ok(self.filesystem.leaf(parent.leaf_id(filename)?).content.clone())
    }

    pub fn get_metadata(&self, path: impl AsRef<Path>) -> Result<&'tree Stat, RepoError> {
        let path = path.as_ref();

        if let Ok(directory) = self.filesystem.root.get_directory(path.as_os_str()) {
            return Ok(&directory.stat);
        }

        Ok(self.inode(path)?.stat(&self.filesystem.leaves))
    }

    pub fn get_subtree(&self, path: impl AsRef<Path>) -> Result<Tree, RepoError> {
        Ok(Tree::from(FileSystem {
            root: self.filesystem.root.get_directory(path.as_ref().as_os_str())?.clone(),
            leaves: self.filesystem.leaves.clone(),
        }))
    }

    pub fn to_dumpfile(&self) -> Result<String, RepoError> {
        let mut dumpfile = Vec::new();
        write_dumpfile(&mut dumpfile, self.filesystem).map_err(|_| RepoError::InvalidDumpfile)?;

        String::from_utf8(dumpfile).map_err(|_| RepoError::InvalidDumpfile)
    }

    pub fn list(&self, path: impl AsRef<Path>) -> Result<Vec<&'tree OsStr>, RepoError> {
        Ok(self
            .filesystem
            .root
            .get_directory(path.as_ref().as_os_str())?
            .sorted_entries()
            .map(|(name, _)| name)
            .collect())
    }

    pub fn check(&self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();

        self.filesystem.root.get_directory(path.as_os_str()).is_ok() || self.inode(path).is_ok()
    }

    pub fn walk(&self, path: impl AsRef<Path>) -> Result<TreeWalk<'tree>, RepoError> {
        let path = path.as_ref();
        let directory = self.filesystem.root.get_directory(path.as_os_str())?;

        Ok(TreeWalk::new(path, directory))
    }

    fn inode(&self, path: &Path) -> Result<&'tree Inode<ObjectID>, RepoError> {
        let (parent, filename) = self.filesystem.root.split(path.as_os_str())?;

        parent.lookup(filename).ok_or(RepoError::NotFound)
    }
}

impl<'tree> TreeWalk<'tree> {
    fn new(path: &Path, directory: &'tree Directory<ObjectID>) -> Self {
        let mut walk = Self { pending: Vec::new() };
        walk.descend(path, directory);

        walk
    }

    fn descend(&mut self, path: &Path, directory: &'tree Directory<ObjectID>) {
        let start = self.pending.len();

        self.pending
            .extend(directory.sorted_entries().map(|(name, inode)| (path.join(name), inode)));
        self.pending[start..].reverse();
    }
}

impl<'tree> Iterator for TreeWalk<'tree> {
    type Item = (PathBuf, &'tree Inode<ObjectID>);

    fn next(&mut self) -> Option<Self::Item> {
        let (path, inode) = self.pending.pop()?;

        if let Inode::Directory(directory) = inode {
            self.descend(&path, directory);
        }

        Some((path, inode))
    }
}

impl<'tree> IntoIterator for TreeView<'tree> {
    type Item = (PathBuf, &'tree Inode<ObjectID>);
    type IntoIter = TreeWalk<'tree>;

    fn into_iter(self) -> Self::IntoIter {
        TreeWalk::new(Path::new(""), &self.filesystem.root)
    }
}
