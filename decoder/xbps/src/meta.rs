// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::HashMap;
use std::mem::take;

use quick_xml::escape::resolve_xml_entity;
use quick_xml::events::Event;
use quick_xml::reader::Reader;

use upac_types::decoder::{DecodeError, DecodeMeta};
use upac_types::{
    DecodedPackageMeta, PackageChecksum, PackageDependency, PackageMeta, Version, VersionConstraint, VersionRequirement,
};

use super::layout::props::{
    ARCH_KEY, ARRAY_TAG, DEPEND_KEY, DESCRIPTION_KEY, KEY_TAG, LICENSE_KEY, MAINTAINER_KEY, NAME_KEY, SIZE_KEY,
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

const OPERATORS: [(&[u8], VersionConstraint); 2] = [
    (b"<=", VersionConstraint::LessOrEqual),
    (b">=", VersionConstraint::GreaterOrEqual),
];

pub struct Props<'content>(pub &'content str);

impl DecodeMeta for Props<'_> {
    fn decode(&self, checksum: PackageChecksum) -> Result<DecodedPackageMeta, DecodeError> {
        let (mut fields, dependencies) = self.parse_fields()?;

        let name = required_field!(fields, NAME_KEY);
        let raw_version = required_field!(fields, VERSION_KEY);

        let installed_size = numeric_field!(fields, SIZE_KEY);

        let meta = PackageMeta {
            name,
            version: Version::parse(&raw_version),
            arch: fields.remove(ARCH_KEY).unwrap_or_default(),
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

impl Props<'_> {
    fn parse_fields(&self) -> Result<(HashMap<String, String>, Vec<PackageDependency>), DecodeError> {
        let mut reader = Reader::from_str(self.0);

        let mut fields = HashMap::new();
        let mut dependencies = Vec::new();
        let mut current_key: Option<String> = None;
        let mut in_array = false;
        let mut buffer = String::new();

        loop {
            match reader.read_event().map_err(|_| DecodeError::MalformedMetadata)? {
                Event::Eof => break,
                Event::Start(tag) if tag.name().as_ref() == ARRAY_TAG.as_bytes() => in_array = true,
                Event::Start(_) => buffer.clear(),
                Event::Text(text) => buffer.push_str(&text),
                Event::GeneralRef(reference) => {
                    if let Some(character) = reference
                        .resolve_char_ref()
                        .map_err(|_| DecodeError::MalformedMetadata)?
                    {
                        buffer.push(character);
                    } else if let Some(resolved) = resolve_xml_entity(&reference) {
                        buffer.push_str(resolved);
                    }
                }
                Event::End(tag) if tag.name().as_ref() == ARRAY_TAG.as_bytes() => {
                    in_array = false;
                    current_key = None;
                }
                Event::End(tag) if tag.name().as_ref() == KEY_TAG.as_bytes() => {
                    current_key = Some(take(&mut buffer));
                }
                Event::End(_) => {
                    let value = take(&mut buffer);

                    if in_array {
                        if current_key.as_deref() == Some(DEPEND_KEY) {
                            dependencies.push(Self::parse_dependency(&value)?);
                        }
                    } else if let Some(key) = current_key.take() {
                        fields.insert(key, value);
                    }
                }
                _ => {}
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
