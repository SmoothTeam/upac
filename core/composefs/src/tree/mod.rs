// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use composefs::erofs::reader::erofs_to_filesystem;
use composefs::generic_tree::{LeafContent, Stat};
use composefs::tree::{FileSystem, Inode, RegularFile};

use self::builder::TreeBuilder;

use crate::error::RepoError;
use crate::{Digest, ObjectID, Repo};

pub mod builder;
pub mod diff;
pub mod overlay;

#[derive(Clone)]
pub struct Tree {
    filesystem: FileSystem<ObjectID>,
}

impl Tree {
    pub fn open(repo: &Repo, digest: &Digest) -> Result<Self, RepoError> {
        let (image, _enable_verity) = repo.repository().open_image(&digest.to_hex())?;

        let mut data = Vec::new();
        File::from(image).read_to_end(&mut data)?;

        Ok(Self {
            filesystem: erofs_to_filesystem(&data)?,
        })
    }

    pub fn edit(&self) -> TreeBuilder {
        TreeBuilder {
            filesystem: self.filesystem.clone(),
        }
    }

    pub fn filesystem(&self) -> &FileSystem<ObjectID> {
        &self.filesystem
    }

    pub fn get(&self, path: impl AsRef<Path>) -> Result<LeafContent<RegularFile<ObjectID>>, RepoError> {
        let (parent, filename) = self.filesystem.root.split(path.as_ref().as_os_str())?;

        Ok(self.filesystem.leaf(parent.leaf_id(filename)?).content.clone())
    }

    pub fn get_metadata(&self, path: impl AsRef<Path>) -> Result<&Stat, RepoError> {
        let path = path.as_ref();

        if let Ok(directory) = self.filesystem.root.get_directory(path.as_os_str()) {
            return Ok(&directory.stat);
        }

        Ok(self.inode(path)?.stat(&self.filesystem.leaves))
    }

    pub fn get_subtree(&self, path: impl AsRef<Path>) -> Result<Tree, RepoError> {
        let mut filesystem = FileSystem {
            root: self.filesystem.root.get_directory(path.as_ref().as_os_str())?.clone(),
            leaves: self.filesystem.leaves.clone(),
        };
        filesystem.compact();

        Ok(Tree { filesystem })
    }

    pub fn list(&self, path: impl AsRef<Path>) -> Result<Vec<&OsStr>, RepoError> {
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

    fn inode(&self, path: &Path) -> Result<&Inode<ObjectID>, RepoError> {
        let (parent, filename) = self.filesystem.root.split(path.as_os_str())?;

        parent.lookup(filename).ok_or(RepoError::NotFound)
    }
}
