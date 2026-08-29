// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::decoder::{CONSTRAINT_ANY, CONSTRAINT_EQUAL, CONSTRAINT_GREATER, CONSTRAINT_LESS};
use upac_types::{Dependency, PackageMeta, Version};

use crate::error::DecodeError;
use crate::header::Header;
use crate::rpm::{
    ARCH_TAG, LICENSE_TAG, NAME_TAG, PACKAGER_TAG, RELEASE_TAG, REQUIRE_FLAGS_TAG, REQUIRE_NAME_TAG,
    REQUIRE_VERSION_TAG, SIZE_TAG, SUMMARY_TAG, URL_TAG, VERSION_TAG,
};

const SENSE_LESS: i32 = 0x02;
const SENSE_GREATER: i32 = 0x04;
const SENSE_EQUAL: i32 = 0x08;
const SENSE_RPMLIB: i32 = 0x0100_0000;

#[derive(Debug)]
pub struct Meta {
    pub meta: PackageMeta,
    pub dependencies: Vec<Dependency>,
}

pub fn build(header: &Header, sha256: [u8; 32]) -> Result<Meta, DecodeError> {
    let name = header.string(NAME_TAG)?.ok_or(DecodeError::MalformedHeader)?;
    let version = header.string(VERSION_TAG)?.ok_or(DecodeError::MalformedHeader)?;

    let raw_version = match header.string(RELEASE_TAG)? {
        Some(release) => format!("{version}-{release}"),
        None => version,
    };

    let arch = header.string(ARCH_TAG)?.ok_or(DecodeError::MalformedHeader)?;
    let installed_size = header.int32(SIZE_TAG)?.unwrap_or(0).max(0) as u64;

    let meta = PackageMeta {
        name,
        version: Version::parse(&raw_version),
        arch,
        arch_sub: None,
        maintainer: header.string(PACKAGER_TAG)?.unwrap_or_default(),
        description: header.string(SUMMARY_TAG)?.unwrap_or_default(),
        license: header.string(LICENSE_TAG)?,
        url: header.string(URL_TAG)?,
        sha256,
        installed_size,
    };

    Ok(Meta {
        meta,
        dependencies: parse_dependencies(header)?,
    })
}

fn parse_dependencies(header: &Header) -> Result<Vec<Dependency>, DecodeError> {
    let names = header.string_array(REQUIRE_NAME_TAG)?;
    let versions = header.string_array(REQUIRE_VERSION_TAG)?;
    let flags = header.int32_array(REQUIRE_FLAGS_TAG)?;

    let mut dependencies = Vec::with_capacity(names.len());
    for (index, name) in names.into_iter().enumerate() {
        let flag = flags.get(index).copied().unwrap_or(0);
        if flag & SENSE_RPMLIB != 0 {
            continue;
        }

        let raw_version = versions.get(index).cloned().unwrap_or_default();

        dependencies.push(Dependency {
            name,
            constraint: sense_to_constraint(flag),
            version: Version::parse(&raw_version),
        });
    }

    Ok(dependencies)
}

fn sense_to_constraint(flag: i32) -> u8 {
    let mut constraint = 0;
    if flag & SENSE_LESS != 0 {
        constraint |= CONSTRAINT_LESS;
    }
    if flag & SENSE_GREATER != 0 {
        constraint |= CONSTRAINT_GREATER;
    }
    if flag & SENSE_EQUAL != 0 {
        constraint |= CONSTRAINT_EQUAL;
    }

    if constraint == 0 { CONSTRAINT_ANY } else { constraint }
}
