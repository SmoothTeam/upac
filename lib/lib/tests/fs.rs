// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs;

use tempfile::tempdir;

use upac::fs::WrittenFile;
use upac::orchestrator::stage::RollbackGuard;

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
fn rollback_restores_the_previous_content() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("existing.txt");
    fs::write(&path, b"before").unwrap();

    let written = WrittenFile::write(&path, b"after").unwrap();
    let mut guard = vec![written];
    guard.rollback().unwrap();

    assert_eq!(fs::read(&path).unwrap(), b"before");
}

#[test]
fn rollback_deletes_a_newly_created_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("new.txt");

    let written = WrittenFile::write(&path, b"content").unwrap();
    let mut guard = vec![written];
    guard.rollback().unwrap();

    assert!(!path.exists());
}

#[test]
fn rollback_undoes_multiple_writes_in_reverse_order() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("existing.txt");
    fs::write(&path, b"original").unwrap();

    let first = WrittenFile::write(&path, b"first").unwrap();
    let second = WrittenFile::write(&path, b"second").unwrap();
    let mut guard = vec![first, second];

    guard.rollback().unwrap();

    assert_eq!(fs::read(&path).unwrap(), b"original");
}
