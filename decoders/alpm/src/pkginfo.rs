// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::HashMap;

use upac_abi::decoder::{CONSTRAINT_ANY, CONSTRAINT_EQUAL, CONSTRAINT_GREATER, CONSTRAINT_LESS};

use upac_types::{Dependency, PackageMeta, Version};

use crate::alpm::{
    PKGINFO_ARCH_KEY, PKGINFO_DEPEND_KEY, PKGINFO_DESCRIPTION_KEY, PKGINFO_EPOCH_KEY, PKGINFO_LICENSE_KEY,
    PKGINFO_MAINTAINER_KEY, PKGINFO_NAME_KEY, PKGINFO_RELEASE_KEY, PKGINFO_SIZE_KEY, PKGINFO_URL_KEY,
    PKGINFO_VERSION_KEY,
};
use crate::error::DecodeError;

pub struct PkgInfo {
    pub meta: PackageMeta,
    pub dependencies: Vec<Dependency>,
}

impl PkgInfo {
    pub fn parse(content: &str, sha256: [u8; 32]) -> Result<PkgInfo, DecodeError> {
        let mut fields: HashMap<&str, String> = HashMap::new();
        let mut dependencies = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let Some((key, value)) = line.split_once(" = ") else {
                continue;
            };

            if key == PKGINFO_DEPEND_KEY {
                dependencies.push(Self::parse_dependency(value));
            } else {
                fields.insert(key, value.to_owned());
            }
        }

        let name = fields.remove(PKGINFO_NAME_KEY).ok_or(DecodeError::MalformedPkgInfo)?;
        let version = fields
            .remove(PKGINFO_VERSION_KEY)
            .ok_or(DecodeError::MalformedPkgInfo)?;

        let raw_version = match fields.remove(PKGINFO_RELEASE_KEY) {
            Some(release) => format!("{version}-{release}"),
            None => version,
        };

        let epoch = fields
            .get(PKGINFO_EPOCH_KEY)
            .and_then(|epoch| epoch.parse().ok())
            .unwrap_or(0);

        let installed_size = fields
            .get(PKGINFO_SIZE_KEY)
            .and_then(|size| size.parse().ok())
            .unwrap_or(0);

        let meta = PackageMeta {
            name,
            version: Version {
                epoch,
                raw: raw_version,
            },
            arch: fields.remove(PKGINFO_ARCH_KEY).unwrap_or_else(|| "any".to_owned()),
            arch_sub: None,
            maintainer: fields.remove(PKGINFO_MAINTAINER_KEY).unwrap_or_default(),
            description: fields.remove(PKGINFO_DESCRIPTION_KEY).unwrap_or_default(),
            license: fields.remove(PKGINFO_LICENSE_KEY),
            url: fields.remove(PKGINFO_URL_KEY),
            sha256,
            installed_size,
        };

        Ok(PkgInfo { meta, dependencies })
    }

    fn parse_dependency(raw: &str) -> Dependency {
        let bytes = raw.as_bytes();

        for index in 0..bytes.len() {
            let Some((constraint, operator_len)) = Self::parse_constraint(&bytes[index..]) else {
                continue;
            };

            return Dependency {
                name: raw[..index].to_owned(),
                constraint,
                version: Version::parse(&raw[index + operator_len..]),
            };
        }

        Dependency {
            name: raw.to_owned(),
            constraint: CONSTRAINT_ANY,
            version: Version::default(),
        }
    }

    fn parse_constraint(token: &[u8]) -> Option<(u8, usize)> {
        match token {
            [b'<', b'=', ..] => Some((CONSTRAINT_LESS | CONSTRAINT_EQUAL, 2)),
            [b'>', b'=', ..] => Some((CONSTRAINT_GREATER | CONSTRAINT_EQUAL, 2)),
            [b'<', ..] => Some((CONSTRAINT_LESS, 1)),
            [b'>', ..] => Some((CONSTRAINT_GREATER, 1)),
            [b'=', ..] => Some((CONSTRAINT_EQUAL, 1)),
            _ => None,
        }
    }
}
