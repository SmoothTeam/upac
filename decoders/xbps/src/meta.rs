// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::HashMap;
use std::mem::take;

use quick_xml::escape::resolve_xml_entity;
use quick_xml::events::Event;
use quick_xml::reader::Reader;

use upac_abi::decoder::{
    CONSTRAINT_ANY, CONSTRAINT_EQUAL, CONSTRAINT_GREATER, CONSTRAINT_LESS, DecodeError, parse_constraint_prefix,
};
use upac_types::decoder::{DecodeMeta, DecodedMeta};
use upac_types::{Dependency, PackageMeta, Version};

use crate::xbps::{
    ARCHITECTURE_KEY, HOMEPAGE_KEY, INSTALLED_SIZE_KEY, LICENSE_KEY, MAINTAINER_KEY, PKGNAME_KEY, RUN_DEPENDS_KEY,
    SHORT_DESC_KEY, VERSION_KEY,
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

const OPERATORS: [(&[u8], u8); 2] = [
    (b"<=", CONSTRAINT_LESS | CONSTRAINT_EQUAL),
    (b">=", CONSTRAINT_GREATER | CONSTRAINT_EQUAL),
];

pub struct Props<'a>(pub &'a str);

impl DecodeMeta for Props<'_> {
    fn decode(&self, sha256: [u8; 32]) -> Result<DecodedMeta, DecodeError> {
        let (mut fields, run_depends) = Self::parse_plist(self.0)?;

        let name = required_field!(fields, PKGNAME_KEY);
        let raw_version = required_field!(fields, VERSION_KEY);

        let installed_size = numeric_field!(fields, INSTALLED_SIZE_KEY);

        let meta = PackageMeta {
            name,
            version: Version::parse(&raw_version),
            arch: fields.remove(ARCHITECTURE_KEY).unwrap_or_default(),
            arch_sub: None,
            maintainer: fields.remove(MAINTAINER_KEY).unwrap_or_default(),
            description: fields.remove(SHORT_DESC_KEY).unwrap_or_default(),
            license: fields.remove(LICENSE_KEY),
            url: fields.remove(HOMEPAGE_KEY),
            sha256,
            installed_size,
        };

        let dependencies = run_depends.iter().map(|dep| Self::parse_dependency(dep)).collect();

        Ok(DecodedMeta { meta, dependencies })
    }
}

impl Props<'_> {
    fn parse_plist(xml: &str) -> Result<(HashMap<String, String>, Vec<String>), DecodeError> {
        let mut reader = Reader::from_str(xml);

        let mut fields = HashMap::new();
        let mut run_depends = Vec::new();
        let mut current_key: Option<String> = None;
        let mut in_array = false;
        let mut buffer = String::new();

        loop {
            match reader.read_event().map_err(|_| DecodeError::MalformedMetadata)? {
                Event::Eof => break,
                Event::Start(tag) if tag.name().as_ref() == "array" => in_array = true,
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
                Event::End(tag) if tag.name().as_ref() == "array" => {
                    in_array = false;
                    current_key = None;
                }
                Event::End(tag) if tag.name().as_ref() == "key" => {
                    current_key = Some(take(&mut buffer));
                }
                Event::End(_) => {
                    let value = take(&mut buffer);

                    if in_array {
                        if current_key.as_deref() == Some(RUN_DEPENDS_KEY) {
                            run_depends.push(value);
                        }
                    } else if let Some(key) = current_key.take() {
                        fields.insert(key, value);
                    }
                }
                _ => {}
            }
        }

        Ok((fields, run_depends))
    }

    fn parse_dependency(raw: &str) -> Dependency {
        let bytes = raw.as_bytes();

        for index in 0..bytes.len() {
            let Some((constraint, operator_len)) = parse_constraint_prefix(&bytes[index..], &OPERATORS) else {
                continue;
            };

            let name = raw[..index].to_owned();
            let version = raw[index + operator_len..].to_owned();

            return Dependency {
                name,
                constraint,
                version: Version::parse(&version),
            };
        }

        Dependency {
            name: raw.to_owned(),
            constraint: CONSTRAINT_ANY,
            version: Version::default(),
        }
    }
}
