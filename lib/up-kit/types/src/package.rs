// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::iter::once;
use std::mem::size_of;
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

use composefs::dumpfile::{write_directory, write_hardlink, write_leaf};
use composefs::dumpfile_parse::{Entry, Item};
use composefs::fsverity::{FsVerityHashValue, Sha256HashValue};
use composefs::generic_tree::{LeafContent, Stat};
use composefs::tree::{Leaf, RegularFile};

use nix::sys::stat::{Mode, SFlag};

use serde::{Deserialize, Serialize};

use sha2::{Digest, Sha256};

use upac_abi::types::{COwned, CSlice, CVec};
use upac_abi::{
    CONSTRAINT_ANY, CONSTRAINT_EQUAL, CONSTRAINT_GREATER, CONSTRAINT_LESS, CPackageChecksum, CPackageDependency,
    CPackageId, CPackageMeta, CPackageTrigger, CVersion,
};

use upac_macro::{CEnum, CTryToRust, RustToC};

use crate::CancelToken;
use crate::decoder::DecodeError;
use crate::error::ErrorKind;

pub type ObjectID = Sha256HashValue;

pub type PackageFiles = Vec<PackageNode>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum VersionToken<'raw> {
    Alpha(&'raw str),
    Numeric(u64),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionRequirement {
    Any,
    Bounded {
        constraint: VersionConstraint,
        version: Version,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionConstraint {
    Less,
    LessOrEqual,
    Equal,
    NotEqual,
    GreaterOrEqual,
    Greater,
}

impl VersionConstraint {
    pub fn from_orderings(less: bool, equal: bool, greater: bool) -> Option<VersionConstraint> {
        match (less, equal, greater) {
            (true, false, false) => Some(VersionConstraint::Less),
            (true, true, false) => Some(VersionConstraint::LessOrEqual),
            (false, true, false) => Some(VersionConstraint::Equal),
            (true, false, true) => Some(VersionConstraint::NotEqual),
            (false, true, true) => Some(VersionConstraint::GreaterOrEqual),
            (false, false, true) => Some(VersionConstraint::Greater),
            _ => None,
        }
    }

    pub fn parse_prefix(token: &[u8], operators: &[(&[u8], VersionConstraint)]) -> Option<(VersionConstraint, usize)> {
        operators
            .iter()
            .find(|(operator, _)| token.starts_with(operator))
            .map(|(operator, constraint)| (*constraint, operator.len()))
    }

    fn from_wire_bits(bits: u8) -> Option<VersionConstraint> {
        VersionConstraint::from_orderings(
            bits & CONSTRAINT_LESS != 0,
            bits & CONSTRAINT_EQUAL != 0,
            bits & CONSTRAINT_GREATER != 0,
        )
    }

    fn wire_bits(self) -> u8 {
        match self {
            VersionConstraint::Less => CONSTRAINT_LESS,
            VersionConstraint::LessOrEqual => CONSTRAINT_LESS | CONSTRAINT_EQUAL,
            VersionConstraint::Equal => CONSTRAINT_EQUAL,
            VersionConstraint::NotEqual => CONSTRAINT_LESS | CONSTRAINT_GREATER,
            VersionConstraint::GreaterOrEqual => CONSTRAINT_GREATER | CONSTRAINT_EQUAL,
            VersionConstraint::Greater => CONSTRAINT_GREATER,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, CEnum)]
pub enum TriggerPosition {
    PreInstall = 0,
    PostInstall = 1,
    PreUpgrade = 2,
    PostUpgrade = 3,
    PreRemove = 4,
    PostRemove = 5,
}

impl TriggerPosition {
    pub const ALL: [TriggerPosition; 6] = [
        TriggerPosition::PreInstall,
        TriggerPosition::PostInstall,
        TriggerPosition::PreUpgrade,
        TriggerPosition::PostUpgrade,
        TriggerPosition::PreRemove,
        TriggerPosition::PostRemove,
    ];
}

impl Display for TriggerPosition {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_str(match self {
            TriggerPosition::PreInstall => "pre-install",
            TriggerPosition::PostInstall => "post-install",
            TriggerPosition::PreUpgrade => "pre-upgrade",
            TriggerPosition::PostUpgrade => "post-upgrade",
            TriggerPosition::PreRemove => "pre-remove",
            TriggerPosition::PostRemove => "post-remove",
        })
    }
}

impl FromStr for TriggerPosition {
    type Err = DecodeError;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        TriggerPosition::ALL
            .into_iter()
            .find(|position| position.to_string() == name)
            .ok_or(DecodeError::MalformedMetadata)
    }
}

pub enum PackageNode {
    Directory(PathBuf, Stat),
    Leaf(PathBuf, Leaf<ObjectID>),
    HardLink(PathBuf, PathBuf),
}

impl PackageNode {
    pub fn parse(line: &str) -> Result<Self, ErrorKind> {
        fn strip_root(path: &Path) -> Result<PathBuf, ErrorKind> {
            let relative = path
                .strip_prefix(Component::RootDir)
                .map_err(|_| ErrorKind::InvalidEntry)?;

            if relative.as_os_str().is_empty() {
                return Err(ErrorKind::InvalidEntry);
            }

            Ok(relative.to_path_buf())
        }

        fn read_stat(entry: &Entry<'_>) -> Result<Stat, ErrorKind> {
            Ok(Stat {
                st_mode: Mode::from_bits_truncate(entry.mode).bits(),
                st_uid: entry.uid,
                st_gid: entry.gid,
                st_mtim_sec: i64::try_from(entry.mtime.sec).map_err(|_| ErrorKind::InvalidEntry)?,
                st_mtim_nsec: u32::try_from(entry.mtime.nsec).map_err(|_| ErrorKind::InvalidEntry)?,
                xattrs: entry
                    .xattrs
                    .iter()
                    .map(|xattr| (Box::from(&*xattr.key), Box::from(&*xattr.value)))
                    .collect(),
            })
        }

        let entry = Entry::parse(line).map_err(|_| ErrorKind::InvalidEntry)?;
        let path = strip_root(&entry.path)?;

        let content = match &entry.item {
            Item::Directory { .. } => return Ok(Self::Directory(path, read_stat(&entry)?)),
            Item::Hardlink { target } => return Ok(Self::HardLink(path, strip_root(target)?)),
            Item::RegularInline { content, size, .. } if content.is_empty() && *size > 0 => {
                LeafContent::Regular(RegularFile::Sparse(*size))
            }
            Item::RegularInline { content, .. } => LeafContent::Regular(RegularFile::Inline(Box::from(&**content))),
            Item::Regular {
                size,
                fsverity_digest,
                path: object_path,
                ..
            } => {
                let object_id = match fsverity_digest {
                    Some(digest) => ObjectID::from_hex(digest),
                    None => ObjectID::from_object_pathname(object_path.as_os_str().as_bytes()),
                }
                .map_err(|_| ErrorKind::InvalidEntry)?;

                LeafContent::Regular(match fsverity_digest {
                    Some(_) => RegularFile::External(object_id, *size),
                    None => RegularFile::ExternalNoVerity(object_id, *size),
                })
            }
            Item::Device { rdev, .. } if SFlag::from_bits_truncate(entry.mode) & SFlag::S_IFMT == SFlag::S_IFBLK => {
                LeafContent::BlockDevice(*rdev)
            }
            Item::Device { rdev, .. } => LeafContent::CharacterDevice(*rdev),
            Item::Symlink { target, .. } => LeafContent::Symlink(Box::from(target.as_os_str())),
            Item::Fifo { .. } => LeafContent::Fifo,
            Item::Socket { .. } => LeafContent::Socket,
        };

        Ok(Self::Leaf(
            path,
            Leaf {
                stat: read_stat(&entry)?,
                content,
            },
        ))
    }

    pub fn path(&self) -> &Path {
        match self {
            Self::Directory(path, _) | Self::Leaf(path, _) | Self::HardLink(path, _) => path,
        }
    }

    pub fn write(&self, dumpfile: &mut String) -> Result<(), ErrorKind> {
        fn join_root(path: &Path) -> PathBuf {
            once(Component::RootDir)
                .chain(
                    path.components()
                        .filter(|component| !matches!(component, Component::CurDir | Component::RootDir)),
                )
                .collect()
        }

        match self {
            Self::Directory(path, stat) => write_directory(dumpfile, &join_root(path), stat, 1),
            Self::Leaf(path, leaf) => write_leaf(dumpfile, &join_root(path), &leaf.stat, &leaf.content, 1),
            Self::HardLink(path, original) => {
                write_hardlink(dumpfile, &join_root(path), join_root(original).as_os_str())
            }
        }
        .map_err(|_| ErrorKind::InvalidEntry)?;

        dumpfile.push('\n');

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, CTryToRust, RustToC, Serialize, Deserialize)]
pub struct Version {
    pub epoch: u32,
    pub raw: String,
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

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageChecksum([u8; 32]);

impl PackageChecksum {
    pub fn compute(package_path: &Path, cancel: &CancelToken) -> Result<Self, DecodeError> {
        let mut reader = BufReader::new(File::open(package_path)?);
        let mut hasher = Sha256::new();

        loop {
            if cancel.is_cancelled() {
                return Err(DecodeError::Cancelled);
            }

            let buffer = reader.fill_buf()?;
            if buffer.is_empty() {
                break;
            }

            hasher.update(buffer);

            let consumed = buffer.len();
            reader.consume(consumed);
        }

        Ok(Self(hasher.finalize().into()))
    }

    pub fn verify(&self, package_path: &Path, cancel: &CancelToken) -> Result<(), DecodeError> {
        if Self::compute(package_path, cancel)? == *self {
            Ok(())
        } else {
            Err(DecodeError::ChecksumMismatch)
        }
    }
}

impl From<PackageChecksum> for CPackageChecksum {
    fn from(checksum: PackageChecksum) -> Self {
        checksum.0
    }
}

impl TryFrom<&CPackageChecksum> for PackageChecksum {
    type Error = ErrorKind;

    fn try_from(checksum: &CPackageChecksum) -> Result<Self, ErrorKind> {
        Ok(Self(*checksum))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageDependency {
    pub name: String,
    pub requirement: VersionRequirement,
}

impl From<PackageDependency> for CPackageDependency {
    fn from(dependency: PackageDependency) -> Self {
        let (constraint, version) = match dependency.requirement {
            VersionRequirement::Any => (
                CONSTRAINT_ANY,
                CVersion {
                    struct_size: size_of::<CVersion>(),
                    epoch: 0,
                    raw: CSlice::from_slice(None),
                },
            ),
            VersionRequirement::Bounded { constraint, version } => (constraint.wire_bits(), CVersion::from(version)),
        };

        CPackageDependency {
            struct_size: size_of::<CPackageDependency>(),
            name: CSlice::from_owned(dependency.name.into_bytes()),
            constraint,
            version,
        }
    }
}

impl TryFrom<&CPackageDependency> for PackageDependency {
    type Error = ErrorKind;

    fn try_from(dependency: &CPackageDependency) -> Result<Self, ErrorKind> {
        unsafe { dependency.validate()? };

        let requirement = match dependency.constraint {
            CONSTRAINT_ANY => VersionRequirement::Any,
            bits => VersionRequirement::Bounded {
                constraint: VersionConstraint::from_wire_bits(bits).ok_or(ErrorKind::InvalidEntry)?,
                version: Version::try_from(&dependency.version)?,
            },
        };

        Ok(PackageDependency {
            name: <&str>::try_from(&dependency.name)?.to_owned(),
            requirement,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CTryToRust, RustToC)]
pub struct PackageTrigger {
    pub position: TriggerPosition,
    pub name: String,
}

#[derive(Debug, Clone, Default, CTryToRust, RustToC, Serialize, Deserialize)]
pub struct PackageMeta {
    pub name: String,
    pub version: Version,
    pub arch: String,
    pub arch_sub: Option<String>,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub maintainer: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub license: Option<String>,
    pub url: Option<String>,
    pub checksum: PackageChecksum,
    pub installed_size: u64,
    #[serde(default)]
    pub dependencies: Vec<PackageDependency>,
    #[serde(skip)]
    pub triggers: Vec<PackageTrigger>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, CTryToRust, RustToC)]
pub struct PackageId {
    pub name: String,
    pub arch: String,
    pub arch_sub: Option<String>,
}

impl From<&PackageMeta> for PackageId {
    fn from(meta: &PackageMeta) -> Self {
        PackageId {
            name: meta.name.clone(),
            arch: meta.arch.clone(),
            arch_sub: meta.arch_sub.clone(),
        }
    }
}

pub struct DecodedPackage {
    pub meta: PackageMeta,
    pub files: PackageFiles,
}
