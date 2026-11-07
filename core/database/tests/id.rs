// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::Path;

use composefs::fsverity::compute_verity;
use composefs::generic_tree::Stat;

use upac_composefs::ObjectID;

use upac_types::package::PackageInfo;

use upac_database::{PackageEntryId, PackageEntryKind, PackageId};

fn stat(mode: u32) -> Stat {
    Stat {
        st_mode: mode,
        ..Stat::uninitialized()
    }
}

fn content(bytes: &[u8]) -> ObjectID {
    compute_verity(bytes)
}

fn package(name: &str, arch: &str, arch_sub: Option<&str>) -> PackageId {
    PackageId::new(&PackageInfo {
        name: name.to_owned(),
        arch: arch.to_owned(),
        arch_sub: arch_sub.map(str::to_owned),
    })
}

#[test]
fn an_entry_id_reads_back_from_its_name() {
    let id = PackageEntryId::new(
        PackageEntryKind::Symlink,
        content(b"target"),
        &stat(0o777),
        Path::new("/cmp/usr/lib/link"),
    );

    assert_eq!(id.to_string().parse::<PackageEntryId>().unwrap(), id);
}

#[test]
fn the_same_content_with_other_permissions_is_another_entry() {
    let place = Path::new("/cmp/usr/bin/tool");
    let readable = PackageEntryId::new(PackageEntryKind::File, content(b"same"), &stat(0o644), place);
    let executable = PackageEntryId::new(PackageEntryKind::File, content(b"same"), &stat(0o755), place);

    assert_ne!(readable, executable);
    assert_eq!(readable.content(), executable.content());
}

#[test]
fn the_same_file_in_another_place_is_another_entry() {
    let first = PackageEntryId::new(
        PackageEntryKind::File,
        content(b""),
        &stat(0o644),
        Path::new("/cmp/usr/lib/python3/a/__init__.py"),
    );
    let second = PackageEntryId::new(
        PackageEntryKind::File,
        content(b""),
        &stat(0o644),
        Path::new("/cmp/usr/lib/python3/b/__init__.py"),
    );

    assert_ne!(first, second);
    assert_eq!(first.content(), second.content());
}

#[test]
fn the_same_file_in_the_same_place_is_the_same_entry() {
    let place = Path::new("/cmp/usr/share/doc/common/LICENSE");
    let from_one_package = PackageEntryId::new(PackageEntryKind::File, content(b"license"), &stat(0o644), place);
    let from_another_package = PackageEntryId::new(PackageEntryKind::File, content(b"license"), &stat(0o644), place);

    assert_eq!(from_one_package, from_another_package);
}

#[test]
fn a_malformed_entry_name_is_rejected() {
    assert!("file-nothex-nothex".parse::<PackageEntryId>().is_err());
    assert!("socket-00-00".parse::<PackageEntryId>().is_err());
    assert!("file".parse::<PackageEntryId>().is_err());
}

#[test]
fn a_package_id_replaces_what_a_directory_name_cannot_hold() {
    assert_eq!(package("lib/c:dev", "x86_64", None).to_string(), "lib_c_dev:x86_64");
}

#[test]
fn a_package_id_reads_back_with_and_without_arch_sub() {
    let plain = package("vim", "x86_64", None);
    let with_sub = package("vim", "x86_64", Some("v3"));

    assert_eq!(plain.to_string(), "vim:x86_64");
    assert_eq!(with_sub.to_string(), "vim:x86_64:v3");
    assert_eq!(plain.to_string().parse::<PackageId>().unwrap(), plain);
    assert_eq!(with_sub.to_string().parse::<PackageId>().unwrap(), with_sub);
}

#[test]
fn a_malformed_package_name_is_rejected() {
    assert!("vim".parse::<PackageId>().is_err());
    assert!("vim::x86_64".parse::<PackageId>().is_err());
    assert!("a:b:c:d".parse::<PackageId>().is_err());
}
