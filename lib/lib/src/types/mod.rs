// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::mem::size_of;

use upac_abi::decoder::CDependency;
use upac_abi::error::ErrorKind;
use upac_abi::package::{CPackageMeta, CUnpackedPackage, CVersion};
use upac_abi::response::{
    CConfigCommitEntry, CDiffConfigFileEntry, CDiffFileEntryCommon, CDiffPackageEntry, CDiffPrefixFileEntry,
    CDiffUntrackedFileEntry, CHistoryEntry, CPrefixEntry, CSearchFileEntry,
};
use upac_abi::types::{CBorrowed, COwned, CSlice, CVec};
use upac_abi::{DiffFileSource, FileDiffKind, PackageDiffKind};
use upac_macro::{CTryToRust, RedbCodec, RustToC};

include!(concat!(env!("OUT_DIR"), "/layout.rs"));

pub mod states;

#[cfg(test)]
mod tests;

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
#[derive(Debug, Clone, PartialEq, Eq, CTryToRust, RedbCodec, RustToC)]
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

impl TryFrom<&CUnpackedPackage> for PackageTemp {
    type Error = ErrorKind;

    fn try_from(package: &CUnpackedPackage) -> Result<Self, ErrorKind> {
        unsafe { package.validate()? };

        let temp_package_path: &str = (&package.temp_path).try_into()?;

        Ok(PackageTemp {
            meta: PackageMeta::try_from(&package.meta)?,
            temp_package_path: temp_package_path.to_owned(),
        })
    }
}

#[derive(Debug, Clone, CTryToRust, RedbCodec, RustToC)]
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

#[derive(Debug, Clone, CTryToRust)]
pub struct Dependency {
    pub name: String,
    pub constraint: u8,
    pub version: Version,
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

// ── SearchFileEntry ─────────────────────────────────────────────────────────
#[derive(Debug, Clone, RustToC)]
pub struct SearchFileEntry {
    pub path: String,
    pub package_name: String,
    pub is_user: bool,
}

// ── PrefixEntry ─────────────────────────────────────────────────────────────
#[derive(Debug, Clone, RustToC)]
pub struct PrefixEntry {
    pub prefix_digest: String,

    pub subject: String,
    pub message: Option<String>,

    pub timestamp: u64,

    pub working_config: Option<String>,
}

// ── ConfigCommitEntry ─────────────────────────────────────────────────────────────
#[derive(Debug, Clone, RustToC)]
pub struct ConfigCommitEntry {
    pub config_digest: String,

    pub subject: String,
    pub message: Option<String>,
}

// ── HistoryEntry ────────────────────────────────────────────────────────────
#[derive(Debug, Clone, RustToC)]
pub struct HistoryEntry {
    pub prefix_digest: String,

    pub subject: String,
    pub message: Option<String>,

    pub timestamp: u64,

    pub working_config: Option<String>,
    pub config_history: Vec<ConfigCommitEntry>,
}

// ── DiffFileEntryCommon ──────────────────────────────────────────────────────
#[derive(Debug, Clone, RustToC)]
pub struct DiffFileEntryCommon {
    pub path: String,
    pub kind: FileDiffKind,
}

// ── DiffPrefixFileEntry ─────────────────────────────────────────────────────
#[derive(Debug, Clone, RustToC)]
pub struct DiffPrefixFileEntry {
    pub common: DiffFileEntryCommon,
    pub source: DiffFileSource,
    pub package_name: String,
    pub is_user: bool,
}

// ── DiffConfigFileEntry ─────────────────────────────────────────────────────
#[derive(Debug, Clone, RustToC)]
pub struct DiffConfigFileEntry {
    pub common: DiffFileEntryCommon,
    pub package_name: Option<String>,
}

// ── DiffPackageEntry ────────────────────────────────────────────────────────
#[derive(Debug, Clone, RustToC)]
pub struct DiffPackageEntry {
    pub name: String,
    pub kind: PackageDiffKind,
    pub version: Version,

    // Only this package's own files. A changed file with no package to
    // attach to is not here — it's in `diff::run()`'s separate
    // unattached-files return value.
    pub files: Vec<DiffPrefixFileEntry>,
}

// ── DiffUntrackedFileEntry ──────────────────────────────────────────────────
// A changed /usr file that belongs to no package at all — not package-owned,
// not attached as a user file. By design this shouldn't normally happen
// (every /usr file is meant to come with a package), but if it does, it's
// surfaced here rather than silently dropped. No package_name: there is none.
#[derive(Debug, Clone, RustToC)]
pub struct DiffUntrackedFileEntry {
    pub common: DiffFileEntryCommon,
    pub source: DiffFileSource,
}

pub struct Targets(pub Vec<PackageEntry>);

impl Targets {
    pub fn entries(&self) -> &[PackageEntry] {
        &self.0
    }
}

pub struct TmpPath(pub String);

as_str_method!(TmpPath);

pub struct Search(pub String);

as_str_method!(Search);

pub struct RequestedPrefixDigest(pub Option<String>);

pub struct RequestedPrefixDigestRange {
    pub from: Option<String>,
    pub to: Option<String>,
}

pub struct RequestedConfigDigestRange {
    pub from: Option<String>,
    pub to: Option<String>,
}

pub struct DiffPackagesSnapshot {
    pub from: Vec<PackageMeta>,
    pub to: Vec<PackageMeta>,
}
