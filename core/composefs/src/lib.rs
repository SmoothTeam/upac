// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};

use composefs::MAX_INLINE_CONTENT;
use composefs::fsverity::{FsVerityHashValue, Sha256HashValue};
use composefs::repository::{GcResult, ImportContext, Repository, RepositoryConfig};
use composefs::tree::RegularFile;

use nix::fcntl::AT_FDCWD;

use serde::de::Error as DeserializeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use self::error::RepoError;

pub mod error;
pub mod fs;
pub mod stat;
pub mod tree;

pub type ObjectID = Sha256HashValue;

mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}

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
    fn serialize<Output: Serializer>(&self, serializer: Output) -> Result<Output::Ok, Output::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<Input: Deserializer<'de>>(deserializer: Input) -> Result<Self, Input::Error> {
        let hex = String::deserialize(deserializer)?;

        Digest::from_hex(&hex).map_err(|_| DeserializeError::custom("invalid composefs digest"))
    }
}

#[derive(Clone)]
pub struct Repo {
    repository: Arc<Repository<ObjectID>>,
    import_context: Arc<Mutex<ImportContext>>,
}

impl Repo {
    pub fn init(path: &Path) -> Result<Self, RepoError> {
        let (repository, _freshly_initialized) = Repository::init_path(AT_FDCWD, path, RepositoryConfig::default())?;

        Ok(Self::from_repository(repository))
    }

    pub fn open(path: &Path) -> Result<Self, RepoError> {
        Ok(Self::from_repository(Repository::open_path(AT_FDCWD, path)?))
    }

    pub fn repository(&self) -> &Repository<ObjectID> {
        &self.repository
    }

    pub fn read(&self, file: &RegularFile<ObjectID>) -> Result<Vec<u8>, RepoError> {
        match file {
            RegularFile::Inline(content) => Ok(content.to_vec()),
            RegularFile::External(object_id, _) | RegularFile::ExternalNoVerity(object_id, _) => {
                Ok(self.repository.read_object(object_id)?)
            }
            RegularFile::Sparse(size) => Ok(vec![0u8; *size as usize]),
        }
    }

    pub fn store(&self, bytes: &[u8]) -> Result<RegularFile<ObjectID>, RepoError> {
        if bytes.len() <= MAX_INLINE_CONTENT {
            return Ok(RegularFile::Inline(bytes.into()));
        }

        Ok(RegularFile::External(
            self.repository.ensure_object(bytes)?,
            bytes.len() as u64,
        ))
    }

    pub fn store_file(&self, file: &File) -> Result<RegularFile<ObjectID>, RepoError> {
        let size = file.metadata()?.len();

        if size <= MAX_INLINE_CONTENT as u64 {
            let mut content = Vec::with_capacity(size as usize);
            let mut reader = file;
            reader.read_to_end(&mut content)?;

            return Ok(RegularFile::Inline(content.into()));
        }

        let mut import_context = self.import_context.lock().unwrap_or_else(PoisonError::into_inner);
        let (object_id, _method) = self
            .repository
            .ensure_object_from_file(file, size, &mut import_context)?;

        Ok(RegularFile::External(object_id, size))
    }

    pub fn gc(&self, roots: &[Digest]) -> Result<GcResult, RepoError> {
        let root_hexes: Vec<String> = roots.iter().map(Digest::to_hex).collect();
        let root_names: Vec<&str> = root_hexes.iter().map(String::as_str).collect();

        Ok(self.repository.gc(&root_names)?)
    }

    fn from_repository(repository: Repository<ObjectID>) -> Self {
        Self {
            repository: Arc::new(repository),
            import_context: Arc::new(Mutex::new(ImportContext::default())),
        }
    }
}
