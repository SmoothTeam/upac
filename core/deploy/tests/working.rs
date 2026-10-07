// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{create_dir_all, write};
use std::path::{Path, PathBuf};

use composefs::repository::{Repository, RepositoryConfig};

use nix::fcntl::AT_FDCWD;

use tempfile::TempDir;

use uuid::Uuid;

use upac_types::CancelToken;
use upac_types::decoder::PackageTriggers;
use upac_types::diff::DiffFileSource;
use upac_types::package::{PackageMeta, Version};
use upac_types::response::entry::FileEntryScope;
use upac_types::transaction::TransactionKind;

use upac_composefs::ObjectID;

use upac_database::files::FileStore;
use upac_database::meta::MetaStore;
use upac_database::transaction::TransactionStore;

use upac_deploy::Sysroot;
use upac_deploy::deployment::PrefixDeploy;
use upac_deploy::deployment::config::ConfigDeploy;
use upac_deploy::error::PrefixEditError;
use upac_deploy::layout::deployment::{DEPLOYS_DIR, REPO_DIR};

fn scratch_root() -> TempDir {
    let scratch = TempDir::new().unwrap();

    create_dir_all(scratch.path().join(DEPLOYS_DIR)).unwrap();
    Repository::<ObjectID>::init_path(
        AT_FDCWD,
        scratch.path().join(REPO_DIR),
        RepositoryConfig::default().set_insecure(),
    )
    .unwrap();

    scratch
}

fn meta(name: &str, version: &str) -> PackageMeta {
    PackageMeta {
        name: name.to_owned(),
        version: Version {
            epoch: 0,
            raw: version.to_owned(),
        },
        arch: "x86_64".to_owned(),
        arch_sub: None,
        maintainer: "JustPav".to_owned(),
        description: "a package".to_owned(),
        license: None,
        url: None,
        sha256: [0; 32],
        installed_size: 100,
    }
}

fn triggers() -> PackageTriggers {
    PackageTriggers {
        format: "alpm".to_owned(),
        triggers: Vec::new(),
    }
}

fn unpacked_package(scratch: &TempDir, files: &[(&str, &[u8])]) -> PathBuf {
    let root = scratch.path().join("unpacked");

    for (path, content) in files {
        let file_path = root.join(path);
        create_dir_all(file_path.parent().unwrap()).unwrap();
        write(file_path, content).unwrap();
    }

    root
}

fn source_file(scratch: &TempDir, content: &[u8]) -> PathBuf {
    let path = scratch.path().join("source");
    write(&path, content).unwrap();

    path
}

#[test]
fn a_package_lands_in_the_prefix_and_its_etc_in_the_defaults() {
    let root = scratch_root();
    let sysroot = Sysroot::open(root.path()).unwrap();
    let package = TempDir::new().unwrap();
    let unpacked = unpacked_package(&package, &[("usr/bin/foo", b"binary"), ("etc/foo.conf", b"config")]);

    let mut working = sysroot.empty_prefix().unwrap();
    let uuid = working
        .add_package(&meta("foo", "1.0"), &triggers(), &unpacked, &CancelToken::new())
        .unwrap();
    let committed = working
        .commit(TransactionKind::Bootstrap, "genesis".to_owned(), None)
        .unwrap();

    let tree = sysroot.repo().open_tree(&committed.digest).unwrap();
    assert_eq!(tree.read_file("bin/foo").unwrap(), b"binary");
    assert_eq!(tree.read_file("etc/foo.conf").unwrap(), b"config");
    assert_eq!(committed.defaults.read_file("foo.conf").unwrap(), b"config");

    let database = sysroot.prefix_database(&committed.digest).unwrap();
    let mut scopes: Vec<_> = database
        .list_package_files(uuid)
        .unwrap()
        .into_iter()
        .map(|entry| (entry.path, entry.scope))
        .collect();
    scopes.sort_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(
        scopes,
        vec![
            ("bin/foo".to_owned(), FileEntryScope::Prefix),
            ("foo.conf".to_owned(), FileEntryScope::Config),
        ]
    );
    assert_eq!(database.get_transaction().unwrap(), Some(committed.transaction));
}

#[test]
fn adding_an_installed_package_again_is_refused() {
    let root = scratch_root();
    let sysroot = Sysroot::open(root.path()).unwrap();
    let package = TempDir::new().unwrap();
    let unpacked = unpacked_package(&package, &[("usr/bin/foo", b"binary")]);

    let mut working = sysroot.empty_prefix().unwrap();
    working
        .add_package(&meta("foo", "1.0"), &triggers(), &unpacked, &CancelToken::new())
        .unwrap();

    assert_eq!(
        working.add_package(&meta("foo", "1.0"), &triggers(), &unpacked, &CancelToken::new()),
        Err(PrefixEditError::PackageExists)
    );
}

#[test]
fn replacing_a_package_drops_the_files_the_new_version_no_longer_ships() {
    let root = scratch_root();
    let sysroot = Sysroot::open(root.path()).unwrap();
    let old_package = TempDir::new().unwrap();
    let new_package = TempDir::new().unwrap();
    let old_unpacked = unpacked_package(&old_package, &[("usr/bin/old", b"old"), ("usr/bin/kept", b"v1")]);
    let new_unpacked = unpacked_package(&new_package, &[("usr/bin/kept", b"v2")]);

    let mut working = sysroot.empty_prefix().unwrap();
    let uuid = working
        .add_package(&meta("foo", "1.0"), &triggers(), &old_unpacked, &CancelToken::new())
        .unwrap();
    working
        .replace_package(
            uuid,
            &meta("foo", "2.0"),
            &triggers(),
            &new_unpacked,
            &CancelToken::new(),
        )
        .unwrap();

    assert_eq!(
        working.database().get_package_meta(uuid).unwrap().unwrap().version.raw,
        "2.0"
    );

    let committed = working
        .commit(TransactionKind::Update, "update foo".to_owned(), None)
        .unwrap();
    let tree = sysroot.repo().open_tree(&committed.digest).unwrap();
    assert!(!tree.contains("bin/old"));
    assert_eq!(tree.read_file("bin/kept").unwrap(), b"v2");
}

#[test]
fn removing_a_package_keeps_user_files_unless_purged() {
    let root = scratch_root();
    let sysroot = Sysroot::open(root.path()).unwrap();
    let package = TempDir::new().unwrap();
    let unpacked = unpacked_package(&package, &[("usr/bin/foo", b"binary")]);
    let user_file = source_file(&package, b"user");

    for purge in [false, true] {
        let mut working = sysroot.empty_prefix().unwrap();
        let uuid = working
            .add_package(&meta("foo", "1.0"), &triggers(), &unpacked, &CancelToken::new())
            .unwrap();
        working
            .attach_file(uuid, DiffFileSource::Config, "/etc/user.conf", &user_file)
            .unwrap();

        working.remove_package(uuid, purge).unwrap();

        let committed = working
            .commit(TransactionKind::Uninstall, "uninstall foo".to_owned(), None)
            .unwrap();
        let tree = sysroot.repo().open_tree(&committed.digest).unwrap();
        assert!(!tree.contains("bin/foo"));
        assert_eq!(tree.contains("etc/user.conf"), !purge);
    }
}

#[test]
fn an_attached_file_is_placed_by_its_system_path_and_detached_again() {
    let root = scratch_root();
    let sysroot = Sysroot::open(root.path()).unwrap();
    let package = TempDir::new().unwrap();
    let unpacked = unpacked_package(&package, &[("usr/bin/foo", b"binary")]);
    let user_file = source_file(&package, b"user");

    let mut working = sysroot.empty_prefix().unwrap();
    let uuid = working
        .add_package(&meta("files", "1.0"), &triggers(), &unpacked, &CancelToken::new())
        .unwrap();

    working
        .attach_file(uuid, DiffFileSource::Prefix, "/usr/share/deep/note", &user_file)
        .unwrap();
    working
        .attach_file(uuid, DiffFileSource::Config, "/etc/note.conf", &user_file)
        .unwrap();
    working
        .detach_file(uuid, DiffFileSource::Config, "/etc/note.conf")
        .unwrap();

    let committed = working
        .commit(TransactionKind::Files, "attach".to_owned(), None)
        .unwrap();
    let tree = sysroot.repo().open_tree(&committed.digest).unwrap();
    assert_eq!(tree.read_file("share/deep/note").unwrap(), b"user");
    assert!(!tree.contains("etc/note.conf"));
}

#[test]
fn a_path_outside_the_scope_directory_is_refused() {
    let root = scratch_root();
    let sysroot = Sysroot::open(root.path()).unwrap();
    let package = TempDir::new().unwrap();
    let user_file = source_file(&package, b"user");

    let mut working = sysroot.empty_prefix().unwrap();

    for (scope, system_path) in [
        (DiffFileSource::Config, "/usr/share/note"),
        (DiffFileSource::Prefix, "/etc/note.conf"),
        (DiffFileSource::Config, "/etc"),
    ] {
        assert_eq!(
            working.attach_file(Uuid::nil(), scope, system_path, &user_file),
            Err(PrefixEditError::OutsideSystemDir)
        );
    }
}

#[test]
fn unowned_files_are_imported_into_the_prefix_root() {
    let root = scratch_root();
    let sysroot = Sysroot::open(root.path()).unwrap();
    let source = TempDir::new().unwrap();
    let system_dir = unpacked_package(&source, &[("lib/systemd/system/setup.service", b"unit")]);

    let mut working = sysroot.empty_prefix().unwrap();
    working.add_unowned_dir(&system_dir, &CancelToken::new()).unwrap();
    working
        .add_unowned_dir(Path::new("/nonexistent/upac/source"), &CancelToken::new())
        .unwrap();

    let committed = working
        .commit(TransactionKind::Bootstrap, "genesis".to_owned(), None)
        .unwrap();
    let tree = sysroot.repo().open_tree(&committed.digest).unwrap();
    assert_eq!(tree.read_file("lib/systemd/system/setup.service").unwrap(), b"unit");
}

#[test]
fn a_working_prefix_builds_on_its_base_transaction() {
    let root = scratch_root();
    let sysroot = Sysroot::open(root.path()).unwrap();

    let genesis = sysroot
        .empty_prefix()
        .unwrap()
        .commit(TransactionKind::Bootstrap, "genesis".to_owned(), None)
        .unwrap();
    let config_digest = sysroot.repo().empty_tree().commit().unwrap();
    let base = PrefixDeploy::new(
        genesis.digest,
        genesis.transaction.clone(),
        ConfigDeploy::new(config_digest, "genesis".to_owned(), None),
    );
    sysroot.create_prefix(&base).unwrap();

    let next = sysroot
        .working_prefix(&base)
        .unwrap()
        .commit(TransactionKind::Install, "install foo".to_owned(), None)
        .unwrap();

    assert_eq!(next.transaction.parent, Some(genesis.transaction.uuid.to_string()));
}

#[test]
fn the_defaults_of_a_prefix_without_etc_are_empty() {
    let root = scratch_root();
    let sysroot = Sysroot::open(root.path()).unwrap();

    let committed = sysroot
        .empty_prefix()
        .unwrap()
        .commit(TransactionKind::Bootstrap, "genesis".to_owned(), None)
        .unwrap();

    let defaults = sysroot.prefix_defaults(&committed.digest).unwrap();
    assert!(!defaults.contains("anything"));
}
