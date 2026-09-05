// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{create_dir_all, write};

use tempfile::TempDir;

use upac_setup::meta::SourceDir;

#[test]
fn read_parses_meta_toml_with_serde_defaults() {
    let scratch = TempDir::new().unwrap();
    write(
        scratch.path().join("meta.toml"),
        "name = \"test-pkg\"\narch = \"x86_64\"\n",
    )
    .unwrap();

    let source = SourceDir { path: scratch.path() };
    let meta = source.read(None).unwrap();

    assert_eq!(meta.name, "test-pkg");
    assert_eq!(meta.arch, "x86_64");
}

#[test]
fn read_honors_explicit_filename_override() {
    let scratch = TempDir::new().unwrap();
    write(scratch.path().join("custom.toml"), "name = \"custom-pkg\"\n").unwrap();

    let source = SourceDir { path: scratch.path() };
    let meta = source.read(Some("custom.toml")).unwrap();

    assert_eq!(meta.name, "custom-pkg");
}

#[test]
fn read_fails_when_file_missing() {
    let scratch = TempDir::new().unwrap();
    let source = SourceDir { path: scratch.path() };

    assert!(source.read(None).is_err());
}

#[test]
fn checksum_sums_installed_size_of_usr_files() {
    let scratch = TempDir::new().unwrap();
    create_dir_all(scratch.path().join("usr")).unwrap();
    write(scratch.path().join("usr/a.txt"), b"12345").unwrap();
    write(scratch.path().join("usr/b.txt"), b"1234567890").unwrap();

    let source = SourceDir { path: scratch.path() };
    let (_, installed_size) = source.checksum(false).unwrap();

    assert_eq!(installed_size, 15);
}

#[test]
fn checksum_excludes_etc_when_include_config_is_false() {
    let scratch = TempDir::new().unwrap();
    create_dir_all(scratch.path().join("usr")).unwrap();
    write(scratch.path().join("usr/a.txt"), b"hello").unwrap();
    create_dir_all(scratch.path().join("etc")).unwrap();
    write(scratch.path().join("etc/b.txt"), b"world").unwrap();

    let source = SourceDir { path: scratch.path() };

    let (hash_without_config, size_without_config) = source.checksum(false).unwrap();
    let (hash_with_config, size_with_config) = source.checksum(true).unwrap();

    assert_ne!(hash_without_config, hash_with_config);
    assert_eq!(size_without_config, 5);
    assert_eq!(size_with_config, 10);
}

#[test]
fn checksum_is_deterministic() {
    let scratch = TempDir::new().unwrap();
    create_dir_all(scratch.path().join("usr")).unwrap();
    write(scratch.path().join("usr/a.txt"), b"hello").unwrap();

    let source = SourceDir { path: scratch.path() };

    let first = source.checksum(false).unwrap();
    let second = source.checksum(false).unwrap();

    assert_eq!(first, second);
}
