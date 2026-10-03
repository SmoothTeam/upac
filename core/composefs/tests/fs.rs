// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;

use tempfile::tempdir;

use upac_composefs::fs::WrittenFile;

#[test]
fn write_creates_a_new_file_with_the_given_content() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("new.txt");

    WrittenFile::write(&path, b"hello").unwrap();

    assert_eq!(fs::read(&path).unwrap(), b"hello");
}

#[test]
fn write_overwrites_an_existing_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("existing.txt");
    fs::write(&path, b"before").unwrap();

    WrittenFile::write(&path, b"after").unwrap();

    assert_eq!(fs::read(&path).unwrap(), b"after");
}

#[test]
fn restore_brings_back_the_previous_content() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("existing.txt");
    fs::write(&path, b"before").unwrap();

    let written = WrittenFile::write(&path, b"after").unwrap();
    written.restore().unwrap();

    assert_eq!(fs::read(&path).unwrap(), b"before");
}

#[test]
fn restore_deletes_a_newly_created_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("new.txt");

    let written = WrittenFile::write(&path, b"content").unwrap();
    written.restore().unwrap();

    assert!(!path.exists());
}

#[test]
fn restoring_multiple_writes_in_reverse_order_brings_back_the_original() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("existing.txt");
    fs::write(&path, b"original").unwrap();

    let first = WrittenFile::write(&path, b"first").unwrap();
    let second = WrittenFile::write(&path, b"second").unwrap();

    second.restore().unwrap();
    first.restore().unwrap();

    assert_eq!(fs::read(&path).unwrap(), b"original");
}

#[test]
fn write_gives_a_new_file_the_configured_permissions() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("new.txt");

    WrittenFile::write(&path, b"hello").unwrap();

    assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o644);
}

#[test]
fn write_keeps_the_permissions_of_the_file_it_replaces() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("existing.txt");
    fs::write(&path, b"before").unwrap();
    fs::set_permissions(&path, Permissions::from_mode(0o640)).unwrap();

    WrittenFile::write(&path, b"after").unwrap();

    assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o640);
}

#[test]
fn write_refuses_a_path_whose_current_content_it_cannot_read() {
    let dir = tempdir().unwrap();

    assert!(WrittenFile::write(dir.path(), b"content").is_err());
    assert!(dir.path().is_dir());
}
