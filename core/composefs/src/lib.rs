// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use composefs::erofs::reader::erofs_to_filesystem;
use composefs::erofs::writer::{ValidatedFileSystem, mkfs_erofs};
use composefs::fsverity::{FsVerityHashValue, Sha256HashValue};
use composefs::generic_tree::Stat;
use composefs::repository::{GcResult, Repository, RepositoryConfig};
use composefs::tree::FileSystem;

use nix::fcntl::AT_FDCWD;

use serde::de::Error as DeserializeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use self::error::RepoError;
use self::tree::Tree;

mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}

pub mod error;
pub mod fs;
pub mod tree;

pub type ObjectID = Sha256HashValue;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Digest(ObjectID);

impl Digest {
    pub fn from_hex(hex: &str) -> Result<Self, RepoError> {
        Ok(Self(ObjectID::from_hex(hex)?))
    }

    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    pub fn object_id(&self) -> &ObjectID {
        &self.0
    }
}

impl From<ObjectID> for Digest {
    fn from(object_id: ObjectID) -> Self {
        Self(object_id)
    }
}

impl Display for Digest {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_str(&self.to_hex())
    }
}

impl Serialize for Digest {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let hex = String::deserialize(deserializer)?;

        Digest::from_hex(&hex).map_err(|_| DeserializeError::custom("invalid composefs digest"))
    }
}

#[derive(Clone)]
pub struct Repo(Arc<Repository<ObjectID>>);

impl Repo {
    pub fn init(path: &Path) -> Result<Self, RepoError> {
        let (repository, _freshly_initialized) = Repository::init_path(AT_FDCWD, path, RepositoryConfig::default())?;

        Ok(Self(Arc::new(repository)))
    }

    pub fn open(path: &Path) -> Result<Self, RepoError> {
        Ok(Self(Arc::new(Repository::open_path(AT_FDCWD, path)?)))
    }

    pub fn upstream(&self) -> &Repository<ObjectID> {
        &self.0
    }

    pub fn empty_tree(&self) -> Tree {
        Tree::new(self.clone(), FileSystem::new(Stat::uninitialized()))
    }

    pub fn open_tree(&self, digest: &Digest) -> Result<Tree, RepoError> {
        let (image, _enable_verity) = self.0.open_image(&digest.to_hex())?;

        let mut data = Vec::new();
        File::from(image).read_to_end(&mut data)?;

        Ok(Tree::new(self.clone(), erofs_to_filesystem(&data)?))
    }

    pub fn gc(&self, roots: &[Digest]) -> Result<GcResult, RepoError> {
        let root_hexes: Vec<String> = roots.iter().map(Digest::to_hex).collect();
        let root_names: Vec<&str> = root_hexes.iter().map(String::as_str).collect();

        Ok(self.0.gc(&root_names)?)
    }

    pub(crate) fn commit_filesystem(&self, filesystem: FileSystem<ObjectID>) -> Result<Digest, RepoError> {
        let validated = ValidatedFileSystem::new(filesystem)?;
        let image = mkfs_erofs(&validated);

        Ok(Digest::from(self.0.write_image(None, &image)?))
    }
}
