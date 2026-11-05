// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::BTreeMap;
use std::ffi::OsStr;

use composefs::generic_tree::{LeafContent, Stat};
use composefs::repository::{Repository, RepositoryConfig};
use composefs::tree::RegularFile;

use nix::fcntl::AT_FDCWD;

use tempfile::{Builder, TempDir};

use upac_composefs::tree::Tree;
use upac_composefs::tree::builder::TreeBuilder;
use upac_composefs::tree::overlay::OverlayBuilder;
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

fn tree_with(repo: &Repo, directories: &[&str], files: &[(&str, &[u8])]) -> Tree {
    let mut builder = TreeBuilder::new(repo);
    for directory in directories {
        builder.insert_dir(directory, Stat::uninitialized()).unwrap();
    }
    for (path, bytes) in files {
        builder.insert(path, file(repo, bytes), Stat::uninitialized()).unwrap();
    }

    builder.commit()
}

#[test]
fn an_upper_file_replaces_the_lower_one() {
    let (_scratch, repo) = open_repo("replace");
    let lower = tree_with(&repo, &[], &[("conf", b"lower")]);
    let upper = tree_with(&repo, &[], &[("conf", b"upper")]);

    let tree = OverlayBuilder::new(&lower, &upper).commit().unwrap();

    assert_eq!(content(&repo, &tree, "conf"), b"upper");
}

#[test]
fn directories_merge_and_keep_the_lower_entries() {
    let (_scratch, repo) = open_repo("merge");
    let lower = tree_with(&repo, &["dir"], &[("dir/kept", b"lower")]);
    let upper = tree_with(&repo, &["dir"], &[("dir/added", b"upper")]);

    let tree = OverlayBuilder::new(&lower, &upper).commit().unwrap();

    assert!(tree.check("dir/kept"));
    assert!(tree.check("dir/added"));
}

#[test]
fn a_whiteout_removes_the_lower_entry() {
    let (_scratch, repo) = open_repo("whiteout");
    let lower = tree_with(&repo, &[], &[("conf", b"lower")]);
    let mut upper = TreeBuilder::new(&repo);
    upper
        .insert("conf", LeafContent::CharacterDevice(0), Stat::uninitialized())
        .unwrap();
    let upper = upper.commit();

    let tree = OverlayBuilder::new(&lower, &upper).commit().unwrap();

    assert!(!tree.check("conf"));
}

#[test]
fn an_opaque_directory_replaces_the_lower_one() {
    let (_scratch, repo) = open_repo("opaque");
    let lower = tree_with(&repo, &["dir"], &[("dir/old", b"lower")]);
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
    let upper = upper.commit();

    let tree = OverlayBuilder::new(&lower, &upper).commit().unwrap();

    assert!(!tree.check("dir/old"));
    assert!(tree.check("dir/new"));
}

#[test]
fn the_lower_root_keeps_its_own_metadata() {
    let (_scratch, repo) = open_repo("target");
    let lower = TreeBuilder::new(&repo).commit();
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
    let upper = upper.commit();

    let tree = OverlayBuilder::new(&lower, &upper).commit().unwrap();

    assert_eq!(tree.get_metadata("").unwrap().st_mode, 0o755);
}
