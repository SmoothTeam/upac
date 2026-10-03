// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, write};

use composefs::generic_tree::Stat;
use composefs::repository::{Repository, RepositoryConfig};

use nix::fcntl::AT_FDCWD;

use tempfile::{Builder, TempDir};

use upac_abi::response::entry::FileDiffKind;

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

fn insert(tree: &mut Tree, path: &str, content: &[u8]) {
    let dir = scratch_dir("diff-source");
    let source_path = dir.path().join("source");
    write(&source_path, content).unwrap();

    tree.insert_file(path, &File::open(&source_path).unwrap(), Stat::uninitialized())
        .unwrap();
}

#[test]
fn run_reports_no_changes_for_identical_trees() {
    let (_scratch, repo) = open_repo("diff-unchanged");
    let mut from = repo.empty_tree();
    let mut to = repo.empty_tree();
    insert(&mut from, "file.txt", b"same");
    insert(&mut to, "file.txt", b"same");

    let changes = from.diff(&to);

    assert!(changes.is_empty());
}

#[test]
fn run_reports_added_for_a_file_only_in_to() {
    let (_scratch, repo) = open_repo("diff-added");
    let from = repo.empty_tree();
    let mut to = repo.empty_tree();
    insert(&mut to, "new.txt", b"content");

    let changes = from.diff(&to);

    assert_eq!(changes, vec![("new.txt".to_owned(), FileDiffKind::Added)]);
}

#[test]
fn run_reports_removed_for_a_file_only_in_from() {
    let (_scratch, repo) = open_repo("diff-removed");
    let mut from = repo.empty_tree();
    let to = repo.empty_tree();
    insert(&mut from, "old.txt", b"content");

    let changes = from.diff(&to);

    assert_eq!(changes, vec![("old.txt".to_owned(), FileDiffKind::Removed)]);
}

#[test]
fn run_reports_modified_for_a_file_with_different_content_in_each_tree() {
    let (_scratch, repo) = open_repo("diff-modified");
    let mut from = repo.empty_tree();
    let mut to = repo.empty_tree();
    insert(&mut from, "file.txt", b"first");
    insert(&mut to, "file.txt", b"second");

    let changes = from.diff(&to);

    assert_eq!(changes, vec![("file.txt".to_owned(), FileDiffKind::Modified)]);
}

#[test]
fn run_recurses_into_matched_subdirectories() {
    let (_scratch, repo) = open_repo("diff-nested");
    let mut from = repo.empty_tree();
    let mut to = repo.empty_tree();
    from.insert_dir("dir", Stat::uninitialized()).unwrap();
    to.insert_dir("dir", Stat::uninitialized()).unwrap();
    insert(&mut to, "dir/new.txt", b"content");

    let changes = from.diff(&to);

    assert_eq!(changes, vec![("dir/new.txt".to_owned(), FileDiffKind::Added)]);
}

#[test]
fn run_marks_both_sides_when_a_directory_is_replaced_by_a_regular_file() {
    let (_scratch, repo) = open_repo("diff-type-change");
    let mut from = repo.empty_tree();
    let mut to = repo.empty_tree();
    from.insert_dir("thing", Stat::uninitialized()).unwrap();
    insert(&mut from, "thing/child", b"content");
    insert(&mut to, "thing", b"content");

    let changes = from.diff(&to);

    assert_eq!(changes.len(), 2);
    assert!(changes.contains(&("thing".to_owned(), FileDiffKind::Added)));
    assert!(changes.contains(&("thing/child".to_owned(), FileDiffKind::Removed)));
}

#[test]
fn run_ignores_a_bare_directory_present_on_only_one_side() {
    let (_scratch, repo) = open_repo("diff-dir-only-side");
    let mut from = repo.empty_tree();
    let mut to = repo.empty_tree();
    to.insert_dir("empty-dir", Stat::uninitialized()).unwrap();
    insert(&mut from, "file.txt", b"content");
    insert(&mut to, "file.txt", b"content");

    let changes = from.diff(&to);

    assert!(changes.is_empty());
}
