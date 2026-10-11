// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::BTreeMap;
use std::path::PathBuf;

use composefs::generic_tree::{LeafContent, Stat};
use composefs::repository::{Repository, RepositoryConfig};
use composefs::tree::RegularFile;

use nix::fcntl::AT_FDCWD;

use tempfile::{Builder, TempDir};

use upac_types::diff::FileDiffKind;

use upac_composefs::tree::Tree;
use upac_composefs::tree::builder::TreeBuilder;
use upac_composefs::{ObjectID, Repo};

fn open_repo(name: &str) -> (TempDir, Repo) {
    let scratch = Builder::new().prefix(name).tempdir().unwrap();
    Repository::<ObjectID>::init_path(AT_FDCWD, scratch.path(), RepositoryConfig::default().set_insecure()).unwrap();
    let repo = Repo::open(scratch.path()).unwrap();

    (scratch, repo)
}

fn tree_with(repo: &Repo, files: &[(&str, &[u8])]) -> Tree {
    let mut builder = TreeBuilder::new(repo);
    for (path, bytes) in files {
        let stored: LeafContent<RegularFile<ObjectID>> = LeafContent::Regular(repo.store(bytes).unwrap());
        builder.insert(path, stored, Stat::uninitialized()).unwrap();
    }

    builder.build()
}

#[test]
fn added_removed_and_modified_files_are_reported() {
    let (_scratch, repo) = open_repo("diff");
    let from = tree_with(&repo, &[("kept", b"same"), ("changed", b"old"), ("gone", b"x")]);
    let to = tree_with(&repo, &[("kept", b"same"), ("changed", b"new"), ("new", b"x")]);

    assert_eq!(
        from.diff(&to),
        BTreeMap::from([
            (PathBuf::from("changed"), FileDiffKind::Modified),
            (PathBuf::from("gone"), FileDiffKind::Removed),
            (PathBuf::from("new"), FileDiffKind::Added),
        ])
    );
}

#[test]
fn a_directory_replaced_by_a_file_marks_both_sides() {
    let (_scratch, repo) = open_repo("diff-replace");
    let from = tree_with(&repo, &[("thing/inner", b"x")]);
    let to = tree_with(&repo, &[("thing", b"x")]);

    assert_eq!(
        from.diff(&to),
        BTreeMap::from([
            (PathBuf::from("thing"), FileDiffKind::Added),
            (PathBuf::from("thing/inner"), FileDiffKind::Removed),
        ])
    );
}
