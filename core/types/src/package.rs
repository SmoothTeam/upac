// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::cmp::Ordering;
use std::mem::size_of;

use serde::{Deserialize, Serialize};

use upac_abi::package::{CPackageDependency, CPackageInfo, CPackageMeta, CVersion};
use upac_abi::types::{COwned, CSlice};
use upac_abi::{CONSTRAINT_ANY, CONSTRAINT_EQUAL, CONSTRAINT_GREATER, CONSTRAINT_LESS};

use upac_macro::{CTryToRust, RustToC};

use crate::error::ErrorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum VersionToken<'raw> {
    Alpha(&'raw str),
    Numeric(u64),
}

#[derive(Debug, Clone, PartialEq, Eq, CTryToRust, RustToC, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, CTryToRust, RustToC, Serialize, Deserialize)]
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

#[derive(Debug, Clone, CTryToRust, RustToC)]
pub struct PackageInfo {
    pub name: String,
    pub arch: String,
    pub arch_sub: Option<String>,
}

impl From<PackageMeta> for PackageInfo {
    fn from(meta: PackageMeta) -> Self {
        let PackageMeta {
            name, arch, arch_sub, ..
        } = meta;
        PackageInfo { name, arch, arch_sub }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionRequirement {
    Any,
    Bounded {
        constraint: VersionConstraint,
        version: Version,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug)]
pub struct DecodedPackageMeta {
    pub meta: PackageMeta,
    pub dependencies: Vec<PackageDependency>,
}
