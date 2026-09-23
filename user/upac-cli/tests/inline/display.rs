// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use upac_types::package::{PackageMeta, Version};

use crate::locale;

use super::{PackageField, PackageFormatter, SizeDisplay, VersionDisplay};

fn meta(name: &str, version_raw: &str, installed_size: u64) -> PackageMeta {
    PackageMeta {
        name: name.to_owned(),
        version: Version {
            epoch: 0,
            raw: version_raw.to_owned(),
        },
        arch: "x86_64".to_owned(),
        maintainer: "someone".to_owned(),
        description: "a package".to_owned(),
        installed_size,
        ..PackageMeta::default()
    }
}

#[test]
fn version_display_omits_a_zero_epoch() {
    let version = Version {
        epoch: 0,
        raw: "1.2.3".to_owned(),
    };

    assert_eq!(VersionDisplay(&version).to_string(), "1.2.3");
}

#[test]
fn version_display_prefixes_a_nonzero_epoch() {
    let version = Version {
        epoch: 2,
        raw: "1.2.3".to_owned(),
    };

    assert_eq!(VersionDisplay(&version).to_string(), "2:1.2.3");
}

#[test]
fn size_display_picks_the_right_unit() {
    assert_eq!(SizeDisplay(512).to_string(), "512 B");
    assert_eq!(SizeDisplay(2048).to_string(), "2 KB");
    assert_eq!(SizeDisplay(5 * 1024 * 1024).to_string(), "5.0 MB");
    assert_eq!(SizeDisplay(3 * 1024 * 1024 * 1024).to_string(), "3.0 GB");
}

#[test]
fn package_field_display_resolves_the_localized_field_name() {
    locale::init_for_test();

    assert_eq!(PackageField::Architecture.display(), "architecture");
    assert_eq!(PackageField::Description.display(), "Description");
}

#[test]
fn field_value_formats_architecture_with_and_without_a_sub_arch() {
    let mut package = meta("upac", "1.0.0", 100);
    assert_eq!(
        PackageFormatter::field_value(&package, PackageField::Architecture),
        "x86_64"
    );

    package.arch_sub = Some("v3".to_owned());
    assert_eq!(
        PackageFormatter::field_value(&package, PackageField::Architecture),
        "x86_64/v3"
    );
}

#[test]
fn field_value_formats_optional_fields_as_empty_when_absent() {
    let package = meta("upac", "1.0.0", 100);

    assert_eq!(PackageFormatter::field_value(&package, PackageField::License), "");
    assert_eq!(PackageFormatter::field_value(&package, PackageField::Url), "");
}

#[test]
fn field_value_formats_the_checksum_as_hex() {
    let mut package = meta("upac", "1.0.0", 100);
    package.sha256 = [0xAB; 32];

    assert_eq!(
        PackageFormatter::field_value(&package, PackageField::Checksum),
        "ab".repeat(32)
    );
}

#[test]
fn field_value_formats_size_and_version() {
    let package = meta("upac", "1.0.0", 2048);

    assert_eq!(PackageFormatter::field_value(&package, PackageField::Size), "2 KB");
    assert_eq!(PackageFormatter::field_value(&package, PackageField::Version), "1.0.0");
}

#[test]
fn ordered_metas_sorts_by_version_when_requested() {
    let metas = [meta("b", "2.0.0", 1), meta("a", "1.0.0", 1)];
    let formatter = PackageFormatter {
        extra_fields: &[],
        metas: &metas,
        sort: Some(PackageField::Version),
    };

    let ordered = formatter.ordered_metas();
    assert_eq!(ordered[0].name, "a");
    assert_eq!(ordered[1].name, "b");
}

#[test]
fn ordered_metas_sorts_by_size_when_requested() {
    let metas = [meta("large", "1.0.0", 1000), meta("small", "1.0.0", 10)];
    let formatter = PackageFormatter {
        extra_fields: &[],
        metas: &metas,
        sort: Some(PackageField::Size),
    };

    let ordered = formatter.ordered_metas();
    assert_eq!(ordered[0].name, "small");
    assert_eq!(ordered[1].name, "large");
}
