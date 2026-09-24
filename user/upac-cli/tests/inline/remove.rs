// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use upac_types::package::PackageMeta;

use crate::locale;

use super::select_installed;

fn meta(name: &str, arch: &str, arch_sub: Option<&str>) -> PackageMeta {
    PackageMeta {
        name: name.to_owned(),
        arch: arch.to_owned(),
        arch_sub: arch_sub.map(str::to_owned),
        ..PackageMeta::default()
    }
}

#[test]
fn select_installed_fails_when_no_package_matches_the_name() {
    locale::init_for_test();
    let installed = [meta("other", "x86_64", None)];

    let error = select_installed("upac", &installed).unwrap_err();

    assert_eq!(error.to_string(), "Package not found: upac");
}

#[test]
fn select_installed_resolves_the_single_match_without_prompting() {
    let installed = [meta("other", "aarch64", None), meta("upac", "x86_64", Some("v3"))];

    let package = select_installed("upac", &installed).unwrap();

    assert_eq!(package.name, "upac");
    assert_eq!(package.arch, "x86_64");
    assert_eq!(package.arch_sub, Some("v3".to_owned()));
}

#[test]
fn select_installed_matches_the_name_exactly() {
    locale::init_for_test();
    let installed = [meta("upac-cli", "x86_64", None)];

    assert!(select_installed("upac", &installed).is_err());
}
