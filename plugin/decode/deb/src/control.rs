// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::HashMap;

use upac_types::decoder::{DecodeError, DecodeMeta};
use upac_types::{
    DecodedPackageMeta, PackageChecksum, PackageDependency, PackageMeta, Version, VersionConstraint, VersionRequirement,
};

use super::layout::control::{
    ARCH_KEY, DEPENDS_KEY, DESCRIPTION_KEY, INSTALLED_SIZE_KEY, MAINTAINER_KEY, NAME_KEY, URL_KEY, VERSION_KEY,
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

const OPERATORS: [(&[u8], VersionConstraint); 5] = [
    (b"<<", VersionConstraint::Less),
    (b"<=", VersionConstraint::LessOrEqual),
    (b">>", VersionConstraint::Greater),
    (b">=", VersionConstraint::GreaterOrEqual),
    (b"=", VersionConstraint::Equal),
];

pub struct ControlFile<'content> {
    pub content: &'content str,
    pub license: Option<String>,
}

impl DecodeMeta for ControlFile<'_> {
    fn decode(&self, checksum: PackageChecksum) -> Result<DecodedPackageMeta, DecodeError> {
        let (mut fields, dependencies) = self.parse_fields()?;

        let name = required_field!(fields, NAME_KEY);
        let raw_version = required_field!(fields, VERSION_KEY);

        let installed_size = numeric_field!(fields, INSTALLED_SIZE_KEY);

        let meta = PackageMeta {
            name,
            version: Version::parse(&raw_version),
            arch: fields.remove(ARCH_KEY).unwrap_or_else(|| "all".to_owned()),
            arch_sub: None,
            maintainer: fields.remove(MAINTAINER_KEY).unwrap_or_default(),
            description: fields.remove(DESCRIPTION_KEY).unwrap_or_default(),
            license: self.license.clone(),
            url: fields.remove(URL_KEY),
            checksum,
            installed_size,
        };

        Ok(DecodedPackageMeta { meta, dependencies })
    }
}

impl ControlFile<'_> {
    fn parse_fields(&self) -> Result<(HashMap<&str, String>, Vec<PackageDependency>), DecodeError> {
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

            if key == DEPENDS_KEY {
                dependencies.extend(Self::parse_depends(value)?);
            } else {
                fields.insert(key, value.to_owned());
            }
        }

        Ok((fields, dependencies))
    }

    fn parse_depends(value: &str) -> Result<Vec<PackageDependency>, DecodeError> {
        value
            .split(',')
            .filter_map(|group| group.split('|').next())
            .map(Self::parse_dependency)
            .collect()
    }

    fn parse_dependency(raw: &str) -> Result<PackageDependency, DecodeError> {
        let raw = raw.trim();
        let bytes = raw.as_bytes();

        for index in 0..bytes.len() {
            let Some((constraint, operator_len)) = VersionConstraint::parse_prefix(&bytes[index..], &OPERATORS) else {
                continue;
            };

            let name = raw[..index].trim().trim_end_matches('(').trim().to_owned();

            let version_part = raw[index + operator_len..].trim();
            let version_str = match version_part.find(')') {
                Some(close_index) => &version_part[..close_index],
                None => version_part,
            };

            let version = Version::parse(version_str);
            if version.raw.is_empty() {
                return Err(DecodeError::MalformedMetadata);
            }

            return Ok(PackageDependency {
                name,
                requirement: VersionRequirement::Bounded { constraint, version },
            });
        }

        Ok(PackageDependency {
            name: raw.to_owned(),
            requirement: VersionRequirement::Any,
        })
    }
}
