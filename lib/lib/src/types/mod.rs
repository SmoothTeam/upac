// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::mem::size_of;

use upac_abi::DiffKind;
use upac_abi::decoder::CDependency;
use upac_abi::error::ErrorKind;
use upac_abi::package::{CPackageMeta, CUnpackedPackage, CVersion};
use upac_abi::response::{
    CCommitEntry, CDiffFileEntry, CDiffPackageEntry, CHistoryEntry, CPrefixEntry, CSearchFileEntry,
};
use upac_abi::types::{CBorrowed, COwned, CSlice, CVec};
use upac_macro::{CTryToRust, RedbCodec, RustToC};

include!(concat!(env!("OUT_DIR"), "/layout.rs"));

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

// ── CommitEntry ─────────────────────────────────────────────────────────────
#[derive(Debug, Clone, RustToC)]
pub struct CommitEntry {
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
    pub config_history: Vec<CommitEntry>,
}

// ── DiffFileEntry ───────────────────────────────────────────────────────────
#[derive(Debug, Clone, RustToC)]
pub struct DiffFileEntry {
    pub path: String,
    pub kind: DiffKind,
    pub package_name: String,
    pub is_user: bool,
}

// ── DiffPackageEntry ────────────────────────────────────────────────────────
#[derive(Debug, Clone, RustToC)]
pub struct DiffPackageEntry {
    pub name: String,
    pub kind: DiffKind,
    pub version: Version,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_version() -> Version {
        Version {
            epoch: 1,
            parts: vec![2, 5, 0],
            pre: Some("rc1".to_owned()),
            release: 3,
        }
    }

    #[test]
    fn version_c_round_trip_preserves_value() {
        let original = sample_version();

        let c_version = CVersion::from(original.clone());
        let restored = Version::try_from(&c_version).unwrap();

        assert_eq!(restored, original);
        unsafe { c_version.free() };
    }

    #[test]
    fn version_redb_round_trip_preserves_value() {
        let original = sample_version();

        let mut buf = Vec::new();
        Version::encode_into(&mut buf, &original);

        let mut offset = 0;
        let restored = Version::decode_from(&buf, &mut offset);

        assert_eq!(restored, original);
        assert_eq!(offset, buf.len());
    }

    #[test]
    fn package_meta_c_round_trip_preserves_value() {
        let original = PackageMeta {
            name: "upac".to_owned(),
            version: sample_version(),
            arch: "x86_64".to_owned(),
            arch_sub: None,
            maintainer: "JustPav".to_owned(),
            description: "package manager".to_owned(),
            license: Some("GPL-3.0-only".to_owned()),
            url: None,
            sha256: [7; 32],
            installed_size: 4096,
        };

        let c_meta = CPackageMeta::from(original.clone());
        let restored = PackageMeta::try_from(&c_meta).unwrap();

        assert_eq!(restored.name, original.name);
        assert_eq!(restored.version, original.version);
        assert_eq!(restored.arch, original.arch);
        assert_eq!(restored.arch_sub, original.arch_sub);
        assert_eq!(restored.maintainer, original.maintainer);
        assert_eq!(restored.description, original.description);
        assert_eq!(restored.license, original.license);
        assert_eq!(restored.url, original.url);
        assert_eq!(restored.sha256, original.sha256);
        assert_eq!(restored.installed_size, original.installed_size);

        unsafe { c_meta.free() };
    }

    #[test]
    fn file_entry_redb_round_trip_preserves_value() {
        let original = FileEntry {
            path: "/usr/bin/up".to_owned(),
            is_user: false,
        };

        let mut buf = Vec::new();
        FileEntry::encode_into(&mut buf, &original);

        let mut offset = 0;
        let restored = FileEntry::decode_from(&buf, &mut offset);

        assert_eq!(restored.path, original.path);
        assert_eq!(restored.is_user, original.is_user);
        assert_eq!(offset, buf.len());
    }
}
