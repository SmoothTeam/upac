// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::BTreeMap;
use std::path::Path;

use composefs::generic_tree::{LeafContent, Stat};
use composefs::tree::{Directory, FileSystem, Inode, RegularFile};

use super::Tree;

use crate::error::RepoError;
use crate::layout::tree::DIR_MODE;
use crate::{Digest, ObjectID, Repo};

mod overlay;

const DIRECTORY_STAT: Stat = Stat {
    st_mode: DIR_MODE,
    st_uid: 0,
    st_gid: 0,
    st_mtim_sec: 0,
    st_mtim_nsec: 0,
    xattrs: BTreeMap::new(),
};

pub struct TreeBuilder {
    pub(super) repo: Repo,
    pub(super) filesystem: FileSystem<ObjectID>,
}

impl TreeBuilder {
    pub fn new(repo: &Repo) -> Self {
        Self {
            repo: repo.clone(),
            filesystem: FileSystem::new(DIRECTORY_STAT),
        }
    }

    pub fn insert(
        &mut self, path: impl AsRef<Path>, content: LeafContent<RegularFile<ObjectID>>, stat: Stat,
    ) -> Result<(), RepoError> {
        let path = path.as_ref();
        self.create_parents(path)?;
        let leaf_id = self.filesystem.push_leaf(stat, content);

        let (parent, filename) = self.filesystem.root.split_mut(path.as_os_str())?;
        parent.insert(filename, Inode::leaf(leaf_id));

        Ok(())
    }

    pub fn insert_dir(&mut self, path: impl AsRef<Path>, stat: Stat) -> Result<(), RepoError> {
        let path = path.as_ref();
        self.create_parents(path)?;

        let (parent, filename) = self.filesystem.root.split_mut(path.as_os_str())?;
        parent.insert(filename, Inode::Directory(Box::new(Directory::new(stat))));

        Ok(())
    }

    pub fn remove(&mut self, path: impl AsRef<Path>) -> Result<(), RepoError> {
        let (parent, filename) = self.filesystem.root.split_mut(path.as_ref().as_os_str())?;
        parent.remove(filename);

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

    pub fn commit(self) -> Result<Digest, RepoError> {
        let TreeBuilder { repo, mut filesystem } = self;
        filesystem.compact();

        Ok(Digest::from(filesystem.commit_image(repo.repository(), None)?))
    }

    pub fn build(self) -> Tree {
        Tree {
            repo: self.repo,
            filesystem: self.filesystem,
        }
    }

    fn create_parents(&mut self, path: &Path) -> Result<(), RepoError> {
        let mut missing: Vec<&Path> = path
            .ancestors()
            .skip(1)
            .filter(|ancestor| !ancestor.as_os_str().is_empty())
            .collect();
        missing.reverse();

        for ancestor in missing {
            if self.filesystem.root.get_directory_opt(ancestor.as_os_str())?.is_none() {
                let (parent, name) = self.filesystem.root.split_mut(ancestor.as_os_str())?;
                parent.insert(name, Inode::Directory(Box::new(Directory::new(DIRECTORY_STAT))));
            }
        }

        Ok(())
    }
}
