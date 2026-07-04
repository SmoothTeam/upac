use std::fs::{File, OpenOptions, create_dir_all};
use std::path::Path;

use nix::fcntl::{Flock, FlockArg};
use ostree::Repo;
use ostree::gio::{Cancellable, File as GioFile};
use serde::{Deserialize, Serialize};

use crate::types::errors::{CommonError, LockError};

include!(concat!(env!("OUT_DIR"), "/layout.rs"));

pub mod errors;
pub mod machine;
pub mod states;

macro_rules! as_str_method {
    ($name:ty) => {
        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

// ── HookResponse ────────────────────────────────────────────────────────────
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookResponse {
    Proceed = 0,
    Cancel = 1,
}

// ── Version ─────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Version {
    pub epoch: u32,
    pub parts: Vec<u32>,
    pub pre: Option<String>,
    pub release: u32,
}

impl Default for Version {
    fn default() -> Self {
        Version {
            epoch: 0,
            parts: Vec::new(),
            pre: None,
            release: 1,
        }
    }
}

// ── Package ─────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct PackageTemp {
    pub meta: PackageMeta,
    pub temp_package_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMeta {
    pub name: String,
    pub version: Version,
    pub arch: String,
    pub arch_sub: Option<String>,
    pub maintainer: String,
    pub description: String,
    pub license: Option<String>,
    pub url: Option<String>,
    pub sha256: [u8; 32],
    pub installed_size: u64,
}

// ── PackageEntry ────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct PackageEntry {
    pub name: String,
    pub arch: String,
    pub arch_sub: Option<String>,
}

// ── FileEntry ───────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub sha256: [u8; 32],
    pub is_user: bool,
}

// ── DiffEntry ───────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub path: String,
    pub kind: DiffKind,
    pub package_name: String,
    pub is_user: bool,
}

// ── FileRecord ──────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct FileRecord {
    pub sha256: [u8; 32],
    pub is_user: bool,
    pub pkg_name: String,
}

// ── PackageRecord ───────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub(crate) struct PackageRecord {
    pub uuid: [u8; 16],
    pub files: Vec<FileEntry>,
    pub name: String,
    pub arch: String,
    pub arch_sub: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffKind {
    Added,
    Removed,
    Modified,
}

pub struct Lock {
    _flock: Flock<File>,
}

impl Lock {
    pub fn acquire() -> Result<Lock, LockError> {
        create_dir_all(runtime::DIR)?;

        let path = Path::new(runtime::DIR).join(runtime::LOCK_FILE);
        let file = OpenOptions::new().create(true).write(true).open(path)?;

        match Flock::lock(file, FlockArg::LockExclusiveNonblock) {
            Ok(flock) => Ok(Lock { _flock: flock }),
            Err((_, errno)) => Err(errno.into()),
        }
    }
}

pub struct RepoHandle {
    repo: Repo,
}

impl RepoHandle {
    pub fn open(path: &str, cancellable: Option<&Cancellable>) -> Result<Self, CommonError> {
        let repo = Repo::new(&GioFile::for_path(path));
        match repo.open(cancellable) {
            Ok(()) => Ok(Self { repo }),
            Err(_) => Err(CommonError::RepoOpenFailed),
        }
    }

    pub fn repo(&self) -> &Repo {
        &self.repo
    }
}

pub struct BaseCommit {
    checksum: String,
}

impl BaseCommit {
    pub fn new(checksum: String) -> Self {
        Self { checksum }
    }

    pub fn as_str(&self) -> &str {
        &self.checksum
    }
}

pub struct Targets(pub Vec<PackageEntry>);

impl Targets {
    pub fn entries(&self) -> &[PackageEntry] {
        &self.0
    }
}

pub struct RepoPath(pub String);

as_str_method!(RepoPath);

pub struct RootPath(pub String);

as_str_method!(RootPath);

pub struct TmpPath(pub String);

as_str_method!(TmpPath);

pub struct Branch(pub String);

as_str_method!(Branch);
