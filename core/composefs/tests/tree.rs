// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsStr;

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
fn insert_creates_the_missing_parents() {
    let (_scratch, repo) = open_repo("parents");
    let mut builder = TreeBuilder::new(&repo);

    builder
        .insert("a/b/file", file(&repo, b"x"), Stat::uninitialized())
        .unwrap();
    let tree = builder.build();

    assert!(tree.check("a/b"));
    assert_eq!(tree.get_metadata("a").unwrap().st_mode, 0o755);
}

#[test]
fn remove_drops_the_whole_subtree() {
    let (_scratch, repo) = open_repo("remove");
    let mut builder = TreeBuilder::new(&repo);
    builder
        .insert("dir/file", file(&repo, b"x"), Stat::uninitialized())
        .unwrap();

    builder.remove("dir").unwrap();

    assert!(!builder.build().check("dir"));
}

#[test]
fn a_committed_tree_reopens_with_its_content() {
    let (_scratch, repo) = open_repo("commit");
    let mut builder = TreeBuilder::new(&repo);
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

    let tree = Tree::open(&repo, &builder.commit().unwrap()).unwrap();

    assert_eq!(content(&repo, &tree, "file"), b"hello");
    assert!(matches!(tree.get("link").unwrap(), LeafContent::Symlink(target) if &*target == "file"));
}

#[test]
fn a_subtree_holds_only_the_requested_directory() {
    let (_scratch, repo) = open_repo("subtree");
    let mut builder = TreeBuilder::new(&repo);
    builder
        .insert("etc/conf", file(&repo, b"conf"), Stat::uninitialized())
        .unwrap();
    builder
        .insert("bin/tool", file(&repo, b"tool"), Stat::uninitialized())
        .unwrap();

    let subtree = builder.build().get_subtree("etc").unwrap();

    assert_eq!(content(&repo, &subtree, "conf"), b"conf");
    assert!(!subtree.check("bin"));
}
