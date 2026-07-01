use serde::{Deserialize, Serialize};

pub mod errors;
pub mod machine;
pub mod states;

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
