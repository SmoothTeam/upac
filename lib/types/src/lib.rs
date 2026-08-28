// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::cmp::Ordering;

use serde::{Deserialize, Deserializer};

use upac_abi::decoder::CDependency;
use upac_abi::error::ErrorKind;
use upac_abi::package::{CPackageMeta, CVersion};
use upac_abi::response::{
    CConfigCommitEntry, CDiffConfigFileEntry, CDiffFileEntryCommon, CDiffPackageEntry, CDiffPrefixFileEntry,
    CDiffUntrackedFileEntry, CHistoryEntry, CPrefixEntry, CSearchFileEntry,
};
use upac_abi::setup::{CBtrfsOptions, CGptLayout, CPartitionMount, CPartitionSpec};
use upac_abi::types::{COwned, CSlice, CVec};
use upac_abi::{DiffFileSource, FileDiffKind, FsKind, PackageDiffKind};

use upac_macro::{CTryToRust, RedbCodec, RustToC};

use crate::codec::RedbCodable;

pub mod codec;
pub mod settings;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum VersionToken<'a> {
    Alpha(&'a str),
    Numeric(u64),
}

#[derive(Debug, Clone, PartialEq, Eq, CTryToRust, RedbCodec, RustToC)]
pub struct Version {
    pub epoch: u32,
    pub raw: String,
}

impl Default for Version {
    fn default() -> Self {
        Version {
            epoch: 0,
            raw: "1.0.0".to_owned(),
        }
    }
}

impl Version {
    pub fn parse(raw: &str) -> Version {
        match raw.split_once(':') {
            Some((epoch, rest)) => Version {
                epoch: epoch.parse().unwrap_or(0),
                raw: rest.to_owned(),
            },
            None => Version {
                epoch: 0,
                raw: raw.to_owned(),
            },
        }
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;

        Ok(Version::parse(&raw))
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.epoch != other.epoch {
            return self.epoch.cmp(&other.epoch);
        }

        let self_tokens = self.tokenize();
        let other_tokens = other.tokenize();

        let mut self_iter = self_tokens.iter();
        let mut other_iter = other_tokens.iter();

        loop {
            match (self_iter.next(), other_iter.next()) {
                (Some(a), Some(b)) => match a.cmp(b) {
                    Ordering::Equal => continue,
                    ordering => return ordering,
                },
                (Some(VersionToken::Numeric(_)), None) => return Ordering::Greater,
                (Some(VersionToken::Alpha(_)), None) => return Ordering::Less,
                (None, Some(VersionToken::Numeric(_))) => return Ordering::Less,
                (None, Some(VersionToken::Alpha(_))) => return Ordering::Greater,
                (None, None) => return Ordering::Equal,
            }
        }
    }
}

impl Version {
    fn tokenize(&self) -> Vec<VersionToken<'_>> {
        let bytes = self.raw.as_bytes();
        let mut tokens = Vec::new();
        let mut index = 0;

        while index < bytes.len() {
            if !bytes[index].is_ascii_alphanumeric() {
                index += 1;
                continue;
            }

            let start = index;
            if bytes[index].is_ascii_digit() {
                while index < bytes.len() && bytes[index].is_ascii_digit() {
                    index += 1;
                }
                let value = self.raw[start..index].parse().unwrap_or(u64::MAX);
                tokens.push(VersionToken::Numeric(value));
            } else {
                while index < bytes.len() && bytes[index].is_ascii_alphabetic() {
                    index += 1;
                }
                tokens.push(VersionToken::Alpha(&self.raw[start..index]));
            }
        }

        tokens
    }
}

// ── Package ─────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct PackageTemp {
    pub meta: PackageMeta,
    pub temp_package_path: String,
}

#[derive(Debug, Clone, RedbCodec)]
pub struct DeclarativeTrigger {
    pub format: String,
    pub triggers: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, CTryToRust, RedbCodec, RustToC)]
#[serde(default)]
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

// ── FileEntryScope ──────────────────────────────────────────────────────────
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileEntryScope {
    Prefix = 0,
    Config = 1,
}

impl RedbCodable for FileEntryScope {
    fn redb_encode(&self, buf: &mut Vec<u8>) {
        buf.push(*self as u8);
    }

    fn redb_decode(data: &[u8], offset: &mut usize) -> FileEntryScope {
        let value = data[*offset];
        *offset += 1;

        match value {
            1 => FileEntryScope::Config,
            _ => FileEntryScope::Prefix,
        }
    }
}

// ── FileEntry ───────────────────────────────────────────────────────────────
#[derive(Debug, Clone, RedbCodec)]
pub struct FileEntry {
    pub path: String,
    pub is_user: bool,
    pub scope: FileEntryScope,
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

// ── PartitionMount / PartitionSpec (bootstrap setup) ────────────────────────
#[derive(Debug, Clone, CTryToRust)]
pub struct PartitionMount {
    pub mount_path: String,
    pub device_path: String,
    pub fs_kind: FsKind,
}

#[derive(Debug, Clone, CTryToRust)]
pub struct PartitionSpec {
    pub mount_path: String,
    pub size_mib: u64,
    pub fs_kind: FsKind,
}

#[derive(Debug, Clone, CTryToRust)]
pub struct GptLayout {
    pub esp_size_mib: u64,
    pub deploy_fs: FsKind,
    pub deploy_size_mib: u64,
    pub extra_partitions: Vec<PartitionSpec>,
}

#[derive(Debug, Clone, CTryToRust)]
pub struct BtrfsOptions {
    pub node_size: u32,
    pub sector_size: u32,
}
