// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, write};
use std::path::PathBuf;

use composefs::generic_tree::Stat;
use composefs::repository::{Repository, RepositoryConfig};

use nix::fcntl::AT_FDCWD;

use tempfile::{Builder, TempDir};

use upac_composefs::tree::Tree;
use upac_composefs::{ObjectID, Repo};

fn scratch_dir(name: &str) -> TempDir {
    Builder::new().prefix(name).tempdir().unwrap()
}

fn open_repo(name: &str) -> (TempDir, Repo) {
    let dir = scratch_dir(name);
    Repository::<ObjectID>::init_path(AT_FDCWD, dir.path(), RepositoryConfig::default().set_insecure()).unwrap();
    let repo = Repo::open(dir.path()).unwrap();

    (dir, repo)
}

fn source_file(label: &str, content: &[u8]) -> File {
    let dir = scratch_dir(label);
    let path = dir.path().join("source");
    write(&path, content).unwrap();

    File::open(&path).unwrap()
}

fn insert(tree: &mut Tree, label: &str, path: &str, content: &[u8]) {
    tree.insert_file(path, &source_file(label, content), Stat::uninitialized())
        .unwrap();
}

fn read(tree: &Tree, path: &str) -> Vec<u8> {
    tree.read_file(path).unwrap()
}

fn exists(tree: &Tree, path: &str) -> bool {
    tree.contains(path)
}

#[test]
fn untouched_file_keeps_the_new_package_default() {
    let (_scratch, repo) = open_repo("untouched");

    let mut base = repo.empty_tree();
    insert(&mut base, "untouched-base", "conf", b"base");
    let live = base.clone();

    let mut new = repo.empty_tree();
    insert(&mut new, "untouched-new", "conf", b"new");

    let (merged_tree, conflicts) = Tree::merge(&base, &new, &live, true).unwrap();

    assert_eq!(read(&merged_tree, "conf"), b"new");
    assert!(conflicts.is_empty());
}

#[test]
fn user_only_edit_is_kept_when_package_did_not_change_the_file() {
    let (_scratch, repo) = open_repo("user-only-edit");

    let mut base = repo.empty_tree();
    insert(&mut base, "user-only-edit-base", "conf", b"base");
    let new = base.clone();

    let mut live = repo.empty_tree();
    insert(&mut live, "user-only-edit-live", "conf", b"user-edit");

    let (merged_tree, conflicts) = Tree::merge(&base, &new, &live, true).unwrap();

    assert_eq!(read(&merged_tree, "conf"), b"user-edit");
    assert!(conflicts.is_empty());
}

#[test]
fn conflicting_edit_keeps_the_user_version_and_writes_upac_new_sidecar() {
    let (_scratch, repo) = open_repo("conflict-edit");

    let mut base = repo.empty_tree();
    insert(&mut base, "conflict-edit-base", "conf", b"base");

    let mut new = repo.empty_tree();
    insert(&mut new, "conflict-edit-new", "conf", b"package-new");

    let mut live = repo.empty_tree();
    insert(&mut live, "conflict-edit-live", "conf", b"user-edit");

    let (merged_tree, conflicts) = Tree::merge(&base, &new, &live, true).unwrap();

    assert_eq!(read(&merged_tree, "conf"), b"user-edit");
    assert_eq!(read(&merged_tree, "conf.upac-new"), b"package-new");
    assert_eq!(conflicts, vec![PathBuf::from("conf")]);
}

#[test]
fn conflicting_edit_skips_the_upac_new_sidecar_when_conflict_files_are_disallowed() {
    let (_scratch, repo) = open_repo("conflict-edit-no-sidecar");

    let mut base = repo.empty_tree();
    insert(&mut base, "conflict-edit-no-sidecar-base", "conf", b"base");

    let mut new = repo.empty_tree();
    insert(&mut new, "conflict-edit-no-sidecar-new", "conf", b"package-new");

    let mut live = repo.empty_tree();
    insert(&mut live, "conflict-edit-no-sidecar-live", "conf", b"user-edit");

    let (merged_tree, conflicts) = Tree::merge(&base, &new, &live, false).unwrap();

    assert_eq!(read(&merged_tree, "conf"), b"user-edit");
    assert!(!exists(&merged_tree, "conf.upac-new"));
    assert_eq!(conflicts, vec![PathBuf::from("conf")]);
}

#[test]
fn user_deletion_is_carried_over_when_the_package_did_not_change_the_file() {
    let (_scratch, repo) = open_repo("user-deletion");

    let mut base = repo.empty_tree();
    insert(&mut base, "user-deletion-base", "conf", b"base");
    let new = base.clone();
    let live = repo.empty_tree();

    let (merged_tree, conflicts) = Tree::merge(&base, &new, &live, true).unwrap();

    assert!(!exists(&merged_tree, "conf"));
    assert!(conflicts.is_empty());
}

#[test]
fn user_deletion_conflicts_when_the_package_also_changed_the_file() {
    let (_scratch, repo) = open_repo("user-deletion-conflict");

    let mut base = repo.empty_tree();
    insert(&mut base, "user-deletion-conflict-base", "conf", b"base");

    let mut new = repo.empty_tree();
    insert(&mut new, "user-deletion-conflict-new", "conf", b"package-new");

    let live = repo.empty_tree();

    let (merged_tree, conflicts) = Tree::merge(&base, &new, &live, true).unwrap();

    assert_eq!(read(&merged_tree, "conf"), b"package-new");
    assert!(!exists(&merged_tree, "conf.upac-new"));
    assert_eq!(conflicts, vec![PathBuf::from("conf")]);
}

#[test]
fn user_edit_survives_when_the_package_stops_providing_the_file() {
    let (_scratch, repo) = open_repo("orphaned-edit");

    let mut base = repo.empty_tree();
    insert(&mut base, "orphaned-edit-base", "conf", b"base");

    let new = repo.empty_tree();

    let mut live = repo.empty_tree();
    insert(&mut live, "orphaned-edit-live", "conf", b"user-edit");

    let (merged_tree, conflicts) = Tree::merge(&base, &new, &live, true).unwrap();

    assert_eq!(read(&merged_tree, "conf"), b"user-edit");
    assert!(!exists(&merged_tree, "conf.upac-new"));
    assert!(conflicts.is_empty());
}

#[test]
fn user_deletion_is_not_a_conflict_when_the_package_also_removed_the_file() {
    let (_scratch, repo) = open_repo("agreed-deletion");

    let mut base = repo.empty_tree();
    insert(&mut base, "agreed-deletion-base", "conf", b"base");

    let new = repo.empty_tree();
    let live = repo.empty_tree();

    let (merged_tree, conflicts) = Tree::merge(&base, &new, &live, true).unwrap();

    assert!(!exists(&merged_tree, "conf"));
    assert!(conflicts.is_empty());
}

#[test]
fn brand_new_user_file_survives_the_merge() {
    let (_scratch, repo) = open_repo("brand-new-user-file");

    let base = repo.empty_tree();
    let new = repo.empty_tree();

    let mut live = repo.empty_tree();
    insert(&mut live, "brand-new-user-file-live", "conf", b"user-only");

    let (merged_tree, conflicts) = Tree::merge(&base, &new, &live, true).unwrap();

    assert_eq!(read(&merged_tree, "conf"), b"user-only");
    assert!(conflicts.is_empty());
}
