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
use upac_composefs::{Digest, ObjectID, Repo};

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

#[test]
fn stored_content_reads_back_inline_and_from_an_object() {
    let (_scratch, repo) = open_repo("store");
    let large = vec![7u8; 4096];

    let small_file = repo.store(b"hello").unwrap();
    let large_file = repo.store(&large).unwrap();

    assert!(matches!(small_file, RegularFile::Inline(_)));
    assert!(matches!(large_file, RegularFile::External(..)));
    assert_eq!(repo.read(&small_file).unwrap(), b"hello");
    assert_eq!(repo.read(&large_file).unwrap(), large);
}

#[test]
fn insert_without_parents_fails() {
    let (_scratch, repo) = open_repo("orphan");
    let mut builder = TreeBuilder::new();

    assert!(
        builder
            .insert("a/file", file(&repo, b"x"), Stat::uninitialized())
            .is_err()
    );
}

#[test]
fn update_replaces_the_content_and_keeps_the_metadata() {
    let (_scratch, repo) = open_repo("update");
    let mut builder = TreeBuilder::new();
    builder
        .insert(
            "file",
            file(&repo, b"old"),
            Stat {
                st_mode: 0o600,
                ..Stat::uninitialized()
            },
        )
        .unwrap();

    builder.update("file", file(&repo, b"new")).unwrap();
    let tree = builder.commit();

    assert_eq!(content(&repo, &tree, "file"), b"new");
    assert_eq!(tree.get_metadata("file").unwrap().st_mode, 0o600);
}

#[test]
fn update_metadata_keeps_the_content() {
    let (_scratch, repo) = open_repo("metadata");
    let mut builder = TreeBuilder::new();
    builder
        .insert("file", file(&repo, b"kept"), Stat::uninitialized())
        .unwrap();

    builder
        .update_metadata(
            "file",
            Stat {
                st_mode: 0o600,
                ..Stat::uninitialized()
            },
        )
        .unwrap();
    let tree = builder.commit();

    assert_eq!(content(&repo, &tree, "file"), b"kept");
    assert_eq!(tree.get_metadata("file").unwrap().st_mode, 0o600);
}

#[test]
fn remove_drops_the_whole_subtree() {
    let (_scratch, repo) = open_repo("remove");
    let mut builder = TreeBuilder::new();
    builder.insert_dir("dir", Stat::uninitialized()).unwrap();
    builder
        .insert("dir/file", file(&repo, b"x"), Stat::uninitialized())
        .unwrap();

    builder.remove("dir").unwrap();

    assert!(!builder.commit().check("dir"));
}

#[test]
fn a_committed_tree_reopens_with_its_content() {
    let (_scratch, repo) = open_repo("commit");
    let mut builder = TreeBuilder::new();
    builder
        .insert("file", file(&repo, b"hello"), Stat::uninitialized())
        .unwrap();
    builder
        .insert(
            "link",
            LeafContent::Symlink(OsStr::new("file").into()),
            Stat::uninitialized(),
        )
        .unwrap();

    let tree = builder.commit();

    assert_eq!(content(&repo, &tree, "file"), b"hello");
    assert!(matches!(tree.get("link").unwrap(), LeafContent::Symlink(target) if &*target == "file"));
}

#[test]
fn a_subtree_holds_only_the_requested_directory() {
    let (_scratch, repo) = open_repo("subtree");
    let mut builder = TreeBuilder::new();
    builder.insert_dir("etc", Stat::uninitialized()).unwrap();
    builder
        .insert("etc/conf", file(&repo, b"conf"), Stat::uninitialized())
        .unwrap();
    builder.insert_dir("bin", Stat::uninitialized()).unwrap();
    builder
        .insert("bin/tool", file(&repo, b"tool"), Stat::uninitialized())
        .unwrap();

    let subtree = builder.commit().get_subtree("etc").unwrap();

    assert_eq!(content(&repo, &subtree, "conf"), b"conf");
    assert!(!subtree.check("bin"));
}

#[test]
fn xattrs_survive_writing_and_reopening_the_image() {
    let (_scratch, repo) = open_repo("xattrs");
    let mut xattrs = BTreeMap::new();
    xattrs.insert(
        OsStr::new("user.upac.entry").into(),
        b"0755 0:0 0 /usr/bin/ping".to_vec().into_boxed_slice(),
    );
    xattrs.insert(
        OsStr::new("security.capability").into(),
        b"capability".to_vec().into_boxed_slice(),
    );
    let mut builder = TreeBuilder::new();
    builder
        .insert(
            "ping",
            file(&repo, b"ping"),
            Stat {
                xattrs: xattrs.clone(),
                ..Stat::uninitialized()
            },
        )
        .unwrap();

    let digest = Digest::from(
        builder
            .commit()
            .filesystem()
            .commit_image(repo.repository(), None)
            .unwrap(),
    );
    let tree = Tree::open(&repo, &digest).unwrap();

    assert_eq!(tree.get_metadata("ping").unwrap().xattrs, xattrs);
}
