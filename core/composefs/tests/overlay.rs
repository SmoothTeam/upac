// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::Path;

use composefs::generic_tree::{LeafContent, Stat};
use composefs::repository::{Repository, RepositoryConfig};
use composefs::tree::RegularFile;

use nix::fcntl::AT_FDCWD;

use tempfile::{Builder, TempDir};

use upac_composefs::tree::Tree;
use upac_composefs::tree::builder::TreeBuilder;
use upac_composefs::{ObjectID, Repo};

fn open_repo(name: &str) -> (TempDir, Repo) {
    let scratch = Builder::new().prefix(name).tempdir().unwrap();
    Repository::<ObjectID>::init_path(AT_FDCWD, scratch.path(), RepositoryConfig::default().set_insecure()).unwrap();
    let repo = Repo::open(scratch.path()).unwrap();

    (scratch, repo)
}

fn file(repo: &Repo, content: &[u8]) -> LeafContent<RegularFile<ObjectID>> {
    LeafContent::Regular(repo.store(content).unwrap())
}

fn content(repo: &Repo, tree: &Tree, path: &str) -> Vec<u8> {
    match tree.get(path).unwrap() {
        LeafContent::Regular(stored) => repo.read(&stored).unwrap(),
        other => panic!("{path} is not a file: {other:?}"),
    }
}

fn tree_with(repo: &Repo, files: &[(&str, &[u8])]) -> Tree {
    let mut builder = TreeBuilder::new(repo);
    for (path, bytes) in files {
        builder.insert(path, file(repo, bytes), Stat::uninitialized()).unwrap();
    }

    builder.build()
}

#[test]
fn an_upper_file_replaces_the_lower_one() {
    let (_scratch, repo) = open_repo("replace");
    let mut lower = tree_with(&repo, &[("conf", b"lower")]).edit();

    lower
        .overlay("", &tree_with(&repo, &[("conf", b"upper")]), None)
        .unwrap();

    assert_eq!(content(&repo, &lower.build(), "conf"), b"upper");
}

#[test]
fn directories_merge_and_keep_the_lower_entries() {
    let (_scratch, repo) = open_repo("merge");
    let mut lower = tree_with(&repo, &[("dir/kept", b"lower")]).edit();

    lower
        .overlay("", &tree_with(&repo, &[("dir/added", b"upper")]), None)
        .unwrap();
    let tree = lower.build();

    assert!(tree.check("dir/kept"));
    assert!(tree.check("dir/added"));
}

#[test]
fn a_whiteout_removes_the_lower_entry() {
    let (_scratch, repo) = open_repo("whiteout");
    let mut lower = tree_with(&repo, &[("conf", b"lower")]).edit();
    let mut upper = TreeBuilder::new(&repo);
    upper
        .insert("conf", LeafContent::CharacterDevice(0), Stat::uninitialized())
        .unwrap();

    lower.overlay("", &upper.build(), None).unwrap();

    assert!(!lower.build().check("conf"));
}

#[test]
fn an_opaque_directory_replaces_the_lower_one() {
    let (_scratch, repo) = open_repo("opaque");
    let mut lower = tree_with(&repo, &[("dir/old", b"lower")]).edit();
    let mut upper = TreeBuilder::new(&repo);
    let mut xattrs = BTreeMap::new();
    xattrs.insert(
        OsStr::new("trusted.overlay.opaque").into(),
        b"y".to_vec().into_boxed_slice(),
    );
    upper
        .insert_dir(
            "dir",
            Stat {
                xattrs,
                ..Stat::uninitialized()
            },
        )
        .unwrap();
    upper
        .insert("dir/new", file(&repo, b"upper"), Stat::uninitialized())
        .unwrap();

    lower.overlay("", &upper.build(), None).unwrap();
    let tree = lower.build();

    assert!(!tree.check("dir/old"));
    assert!(tree.check("dir/new"));
}

#[test]
fn the_target_path_keeps_its_own_metadata() {
    let (_scratch, repo) = open_repo("target");
    let mut lower = TreeBuilder::new(&repo);
    let mut upper = TreeBuilder::new(&repo);
    upper
        .update_metadata(
            "",
            Stat {
                st_mode: 0o700,
                ..Stat::uninitialized()
            },
        )
        .unwrap();

    lower.overlay("", &upper.build(), None).unwrap();

    assert_eq!(lower.build().get_metadata("").unwrap().st_mode, 0o755);
}

#[test]
fn a_single_file_overlays_under_another_name() {
    let (_scratch, repo) = open_repo("single");
    let mut lower = TreeBuilder::new(&repo);

    lower
        .overlay(
            "conf.upac-new",
            &tree_with(&repo, &[("conf", b"package")]),
            Some(Path::new("conf")),
        )
        .unwrap();

    assert_eq!(content(&repo, &lower.build(), "conf.upac-new"), b"package");
}
