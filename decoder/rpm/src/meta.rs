// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::decoder::DecodeError;
use upac_types::package::{
    DecodedPackageMeta, PackageChecksum, PackageDependency, PackageMeta, Version, VersionConstraint, VersionRequirement,
};
use upac_types::traits::DecodeMeta;

use super::header::Header;
use super::layout::tags::{
    ARCH as ARCH_TAG, LICENSE as LICENSE_TAG, NAME as NAME_TAG, PACKAGER as PACKAGER_TAG, RELEASE as RELEASE_TAG,
    REQUIRE_FLAGS as REQUIRE_FLAGS_TAG, REQUIRE_NAME as REQUIRE_NAME_TAG, REQUIRE_VERSION as REQUIRE_VERSION_TAG,
    SIZE as SIZE_TAG, SUMMARY as SUMMARY_TAG, URL as URL_TAG, VERSION as VERSION_TAG,
};

const SENSE_LESS: i32 = 0x02;
const SENSE_GREATER: i32 = 0x04;
const SENSE_EQUAL: i32 = 0x08;
const SENSE_RPMLIB: i32 = 0x0100_0000;

impl DecodeMeta for Header {
    fn decode(&self, checksum: PackageChecksum) -> Result<DecodedPackageMeta, DecodeError> {
        let name = self.string(NAME_TAG)?.ok_or(DecodeError::MalformedMetadata)?;
        let version = self.string(VERSION_TAG)?.ok_or(DecodeError::MalformedMetadata)?;

        let raw_version = match self.string(RELEASE_TAG)? {
            Some(release) => format!("{version}-{release}"),
            None => version,
        };

        let arch = self.string(ARCH_TAG)?.ok_or(DecodeError::MalformedMetadata)?;
        let installed_size = self.int32(SIZE_TAG)?.unwrap_or(0).max(0) as u64;

        let meta = PackageMeta {
            name,
            version: Version::parse(&raw_version),
            arch,
            arch_sub: None,
            maintainer: self.string(PACKAGER_TAG)?.unwrap_or_default(),
            description: self.string(SUMMARY_TAG)?.unwrap_or_default(),
            license: self.string(LICENSE_TAG)?,
            url: self.string(URL_TAG)?,
            checksum,
            installed_size,
        };

        Ok(DecodedPackageMeta {
            meta,
            dependencies: self.parse_dependencies()?,
        })
    }
}

impl Header {
    fn parse_dependencies(&self) -> Result<Vec<PackageDependency>, DecodeError> {
        let names = self.string_array(REQUIRE_NAME_TAG)?;
        let versions = self.string_array(REQUIRE_VERSION_TAG)?;
        let flags = self.int32_array(REQUIRE_FLAGS_TAG)?;

        let mut dependencies = Vec::with_capacity(names.len());
        for (index, name) in names.into_iter().enumerate() {
            let flag = flags.get(index).copied().unwrap_or(0);
            if flag & SENSE_RPMLIB != 0 {
                continue;
            }

            let constraint = VersionConstraint::from_orderings(
                flag & SENSE_LESS != 0,
                flag & SENSE_EQUAL != 0,
                flag & SENSE_GREATER != 0,
            );

            let requirement = match (constraint, versions.get(index)) {
                (Some(constraint), Some(raw_version)) if !raw_version.is_empty() => VersionRequirement::Bounded {
                    constraint,
                    version: Version::parse(raw_version),
                },
                _ => VersionRequirement::Any,
            };

            dependencies.push(PackageDependency { name, requirement });
        }

        Ok(dependencies)
    }
}
