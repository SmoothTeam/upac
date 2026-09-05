// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use crate::commands::display::PackageField;

use super::{Args, build_extra_fields};

fn no_flags() -> Args {
    Args {
        query: String::new(),
        package: None,
        package_arch: None,
        package_arch_sub: None,
        version: false,
        arch: false,
        author: false,
        license: false,
        url: false,
        packager: false,
        size: false,
        description: false,
        checksum: false,
        regex: false,
        sort: None,
    }
}

fn field_bytes(fields: &[PackageField]) -> Vec<u8> {
    fields.iter().map(|field| *field as u8).collect()
}

#[test]
fn build_extra_fields_is_empty_when_no_flags_are_set() {
    assert!(build_extra_fields(&no_flags()).is_empty());
}

#[test]
fn build_extra_fields_follows_a_fixed_order_regardless_of_flag_order() {
    let args = Args {
        checksum: true,
        version: true,
        author: true,
        ..no_flags()
    };

    let expected = field_bytes(&[PackageField::Version, PackageField::Author, PackageField::Checksum]);
    assert_eq!(field_bytes(&build_extra_fields(&args)), expected);
}

#[test]
fn build_extra_fields_includes_every_flag_when_all_are_set() {
    let args = Args {
        version: true,
        arch: true,
        author: true,
        license: true,
        url: true,
        packager: true,
        size: true,
        description: true,
        checksum: true,
        ..no_flags()
    };

    let expected = field_bytes(&[
        PackageField::Version,
        PackageField::Architecture,
        PackageField::Author,
        PackageField::License,
        PackageField::Url,
        PackageField::Packager,
        PackageField::Size,
        PackageField::Description,
        PackageField::Checksum,
    ]);
    assert_eq!(field_bytes(&build_extra_fields(&args)), expected);
}
