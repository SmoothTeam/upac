// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::HashMap;

use upac_abi::decoder::{
    CONSTRAINT_ANY, CONSTRAINT_EQUAL, CONSTRAINT_GREATER, CONSTRAINT_LESS, parse_constraint_prefix,
};

use upac_types::{Dependency, PackageMeta, Version};

use crate::deb::{
    CONTROL_ARCH_KEY, CONTROL_DEPENDS_KEY, CONTROL_DESCRIPTION_KEY, CONTROL_INSTALLED_SIZE_KEY, CONTROL_MAINTAINER_KEY,
    CONTROL_NAME_KEY, CONTROL_URL_KEY, CONTROL_VERSION_KEY,
};
use crate::error::DecodeError;

const OPERATORS: [(&[u8], u8); 5] = [
    (b"<<", CONSTRAINT_LESS),
    (b"<=", CONSTRAINT_LESS | CONSTRAINT_EQUAL),
    (b">>", CONSTRAINT_GREATER),
    (b">=", CONSTRAINT_GREATER | CONSTRAINT_EQUAL),
    (b"=", CONSTRAINT_EQUAL),
];

#[derive(Debug)]
pub struct ControlFile {
    pub meta: PackageMeta,
    pub dependencies: Vec<Dependency>,
}

impl ControlFile {
    pub fn parse(content: &str, license: Option<String>, sha256: [u8; 32]) -> Result<ControlFile, DecodeError> {
        let mut fields: HashMap<&str, String> = HashMap::new();
        let mut dependencies = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let Some((key, value)) = line.split_once(": ") else {
                continue;
            };

            if key == CONTROL_DEPENDS_KEY {
                dependencies.extend(Self::parse_depends(value));
            } else {
                fields.insert(key, value.to_owned());
            }
        }

        let name = fields.remove(CONTROL_NAME_KEY).ok_or(DecodeError::MalformedControl)?;
        let raw_version = fields
            .remove(CONTROL_VERSION_KEY)
            .ok_or(DecodeError::MalformedControl)?;

        let installed_size = fields
            .get(CONTROL_INSTALLED_SIZE_KEY)
            .and_then(|size| size.parse().ok())
            .unwrap_or(0);

        let meta = PackageMeta {
            name,
            version: Version::parse(&raw_version),
            arch: fields.remove(CONTROL_ARCH_KEY).unwrap_or_else(|| "all".to_owned()),
            arch_sub: None,
            maintainer: fields.remove(CONTROL_MAINTAINER_KEY).unwrap_or_default(),
            description: fields.remove(CONTROL_DESCRIPTION_KEY).unwrap_or_default(),
            license,
            url: fields.remove(CONTROL_URL_KEY),
            sha256,
            installed_size,
        };

        Ok(ControlFile { meta, dependencies })
    }

    fn parse_depends(value: &str) -> Vec<Dependency> {
        value
            .split(',')
            .filter_map(|group| group.split('|').next())
            .map(Self::parse_dependency)
            .collect()
    }

    fn parse_dependency(raw: &str) -> Dependency {
        let raw = raw.trim();
        let bytes = raw.as_bytes();

        for index in 0..bytes.len() {
            let Some((constraint, operator_len)) = parse_constraint_prefix(&bytes[index..], &OPERATORS) else {
                continue;
            };

            let name = raw[..index].trim().trim_end_matches('(').trim().to_owned();

            let version_part = raw[index + operator_len..].trim();
            let version_str = match version_part.find(')') {
                Some(close_index) => &version_part[..close_index],
                None => version_part,
            };

            return Dependency {
                name,
                constraint,
                version: Version::parse(version_str),
            };
        }

        Dependency {
            name: raw.to_owned(),
            constraint: CONSTRAINT_ANY,
            version: Version::default(),
        }
    }
}
