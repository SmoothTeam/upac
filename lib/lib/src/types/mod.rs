use std::os::fd::{AsRawFd, OwnedFd};

use nix::sys::socket::{AddressFamily, SockFlag, SockType, UnixAddr, bind, socket};
use upac_macro::RedbCodec;

use crate::types::errors::LockError;

include!(concat!(env!("OUT_DIR"), "/layout.rs"));

pub mod deploy;
pub mod errors;
pub mod hooks;
pub mod machine;
pub mod ostree;
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

// ── Version ─────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq, Eq, RedbCodec)]
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

#[derive(Debug, Clone, RedbCodec)]
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
#[derive(Debug, Clone, RedbCodec)]
pub struct FileEntry {
    pub path: String,
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
    _socket: OwnedFd,
}

impl Lock {
    pub fn acquire() -> Result<Lock, LockError> {
        let socket = socket(AddressFamily::Unix, SockType::Stream, SockFlag::SOCK_CLOEXEC, None)?;
        let address = UnixAddr::new_abstract(runtime::LOCK_ADDRESS.as_bytes())?;

        bind(socket.as_raw_fd(), &address)?;

        Ok(Lock { _socket: socket })
    }
}

pub struct Targets(pub Vec<PackageEntry>);

impl Targets {
    pub fn entries(&self) -> &[PackageEntry] {
        &self.0
    }
}

pub struct TmpPath(pub String);

as_str_method!(TmpPath);

pub struct Branch(pub String);

as_str_method!(Branch);
