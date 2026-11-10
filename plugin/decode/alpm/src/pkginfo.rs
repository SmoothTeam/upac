// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::HashMap;

use upac_types::decoder::{DecodeError, DecodeMeta};
use upac_types::{
    DecodedPackageMeta, PackageChecksum, PackageDependency, PackageMeta, Version, VersionConstraint, VersionRequirement,
};

use super::layout::pkginfo::{
    ARCH_KEY, DEPEND_KEY, DESCRIPTION_KEY, EPOCH_KEY, LICENSE_KEY, MAINTAINER_KEY, NAME_KEY, RELEASE_KEY, SIZE_KEY,
    URL_KEY, VERSION_KEY,
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
    (b"<=", VersionConstraint::LessOrEqual),
    (b">=", VersionConstraint::GreaterOrEqual),
    (b"<", VersionConstraint::Less),
    (b">", VersionConstraint::Greater),
    (b"=", VersionConstraint::Equal),
];

pub struct PkgInfo<'content>(pub &'content str);

impl DecodeMeta for PkgInfo<'_> {
    fn decode(&self, checksum: PackageChecksum) -> Result<DecodedPackageMeta, DecodeError> {
        let (mut fields, dependencies) = self.parse_fields()?;

        let name = required_field!(fields, NAME_KEY);
        let version = required_field!(fields, VERSION_KEY);

        let raw_version = match fields.remove(RELEASE_KEY) {
            Some(release) => format!("{version}-{release}"),
            None => version,
        };

        let epoch = numeric_field!(fields, EPOCH_KEY);
        let installed_size = numeric_field!(fields, SIZE_KEY);

        let meta = PackageMeta {
            name,
            version: Version {
                epoch,
                raw: raw_version,
            },
            arch: fields.remove(ARCH_KEY).unwrap_or_else(|| "any".to_owned()),
            arch_sub: None,
            maintainer: fields.remove(MAINTAINER_KEY).unwrap_or_default(),
            description: fields.remove(DESCRIPTION_KEY).unwrap_or_default(),
            license: fields.remove(LICENSE_KEY),
            url: fields.remove(URL_KEY),
            checksum,
            installed_size,
        };

        Ok(DecodedPackageMeta { meta, dependencies })
    }
}

impl PkgInfo<'_> {
    fn parse_fields(&self) -> Result<(HashMap<&str, String>, Vec<PackageDependency>), DecodeError> {
        let mut fields: HashMap<&str, String> = HashMap::new();
        let mut dependencies = Vec::new();

        for line in self.0.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let Some((key, value)) = line.split_once(" = ") else {
                continue;
            };

            if key == DEPEND_KEY {
                dependencies.push(Self::parse_dependency(value)?);
            } else {
                fields.insert(key, value.to_owned());
            }
        }

        Ok((fields, dependencies))
    }

    fn parse_dependency(value: &str) -> Result<PackageDependency, DecodeError> {
        let bytes = value.as_bytes();

        for index in 0..bytes.len() {
            let Some((constraint, operator_len)) = VersionConstraint::parse_prefix(&bytes[index..], &OPERATORS) else {
                continue;
            };

            let version = Version::parse(&value[index + operator_len..]);
            if version.raw.is_empty() {
                return Err(DecodeError::MalformedMetadata);
            }

            return Ok(PackageDependency {
                name: value[..index].to_owned(),
                requirement: VersionRequirement::Bounded { constraint, version },
            });
        }

        Ok(PackageDependency {
            name: value.to_owned(),
            requirement: VersionRequirement::Any,
        })
    }
}
