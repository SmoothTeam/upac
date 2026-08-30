// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::HashMap;

use upac_abi::decoder::{
    CONSTRAINT_ANY, CONSTRAINT_EQUAL, CONSTRAINT_GREATER, CONSTRAINT_LESS, DecodeError, parse_constraint_prefix,
};
use upac_types::decoder::{DecodeMeta, DecodedMeta};
use upac_types::{Dependency, PackageMeta, Version};

use crate::deb::{
    CONTROL_ARCH_KEY, CONTROL_DEPENDS_KEY, CONTROL_DESCRIPTION_KEY, CONTROL_INSTALLED_SIZE_KEY, CONTROL_MAINTAINER_KEY,
    CONTROL_NAME_KEY, CONTROL_URL_KEY, CONTROL_VERSION_KEY,
};

macro_rules! required_field {
    ($fields:expr, $key:expr) => {
        $fields.remove($key).ok_or(DecodeError::MalformedMetadata)?
    };
}

macro_rules! numeric_field {
    ($fields:expr, $key:expr) => {
        $fields
            .get($key)
            .and_then(|value| value.parse().ok())
            .unwrap_or_default()
    };
}

const OPERATORS: [(&[u8], u8); 5] = [
    (b"<<", CONSTRAINT_LESS),
    (b"<=", CONSTRAINT_LESS | CONSTRAINT_EQUAL),
    (b">>", CONSTRAINT_GREATER),
    (b">=", CONSTRAINT_GREATER | CONSTRAINT_EQUAL),
    (b"=", CONSTRAINT_EQUAL),
];

pub struct ControlFile<'a> {
    pub content: &'a str,
    pub license: Option<String>,
}

impl DecodeMeta for ControlFile<'_> {
    fn decode(&self, sha256: [u8; 32]) -> Result<DecodedMeta, DecodeError> {
        let (mut fields, dependencies) = self.parse_fields();

        let name = required_field!(fields, CONTROL_NAME_KEY);
        let raw_version = required_field!(fields, CONTROL_VERSION_KEY);

        let installed_size = numeric_field!(fields, CONTROL_INSTALLED_SIZE_KEY);

        let meta = PackageMeta {
            name,
            version: Version::parse(&raw_version),
            arch: fields.remove(CONTROL_ARCH_KEY).unwrap_or_else(|| "all".to_owned()),
            arch_sub: None,
            maintainer: fields.remove(CONTROL_MAINTAINER_KEY).unwrap_or_default(),
            description: fields.remove(CONTROL_DESCRIPTION_KEY).unwrap_or_default(),
            license: self.license.clone(),
            url: fields.remove(CONTROL_URL_KEY),
            sha256,
            installed_size,
        };

        Ok(DecodedMeta { meta, dependencies })
    }
}

impl ControlFile<'_> {
    fn parse_fields(&self) -> (HashMap<&str, String>, Vec<Dependency>) {
        let mut fields: HashMap<&str, String> = HashMap::new();
        let mut dependencies = Vec::new();

        for line in self.content.lines() {
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

        (fields, dependencies)
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
