// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use super::has_extension;

#[test]
fn a_multi_part_extension_matches_the_whole_suffix() {
    assert!(has_extension("acl-2.4.0-1-x86_64.pkg.tar.zst", "pkg.tar.zst"));
    assert!(has_extension("zlib-1:1.3.2-3-x86_64.pkg.tar.zst", "pkg.tar.zst"));
}

#[test]
fn a_shorter_extension_does_not_match_a_longer_suffix() {
    assert!(!has_extension("acl-2.4.0-1-x86_64.pkg.tar.zst", "pkg.tar"));
}

#[test]
fn a_single_extension_still_matches() {
    assert!(has_extension("bash_5.2-1_amd64.deb", "deb"));
}

#[test]
fn the_extension_must_follow_a_dot() {
    assert!(!has_extension("notadeb", "deb"));
    assert!(!has_extension("deb", "deb"));
}
