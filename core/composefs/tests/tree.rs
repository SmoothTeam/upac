// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, create_dir_all, read, read_link, write};
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use composefs::generic_tree::Stat;
use composefs::repository::{Repository, RepositoryConfig};

use nix::fcntl::AT_FDCWD;

use tempfile::{Builder, TempDir};

use upac_composefs::error::RepoError;
use upac_composefs::{Digest, ObjectID, Repo};

use upac_types::CancelToken;

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

#[test]
fn an_inserted_directory_is_found() {
    let (_scratch, repo) = open_repo("insert-dir");
    let mut tree = repo.empty_tree();

    tree.insert_dir("dir", Stat::uninitialized()).unwrap();

    assert!(tree.contains("dir"));
    assert!(tree.stat("dir").is_ok());
}

#[test]
fn a_missing_path_is_not_found() {
    let (_scratch, repo) = open_repo("missing");
    let tree = repo.empty_tree();

    assert!(!tree.contains("missing"));
    assert!(matches!(tree.stat("missing"), Err(RepoError::NotFound)));
}

#[test]
fn removing_drops_the_entry() {
    let (_scratch, repo) = open_repo("remove");
    let mut tree = repo.empty_tree();
    tree.insert_dir("dir", Stat::uninitialized()).unwrap();

    tree.remove("dir").unwrap();

    assert!(!tree.contains("dir"));
}

#[test]
fn a_small_file_round_trips_inline() {
    let (_scratch, repo) = open_repo("insert-inline");
    let mut tree = repo.empty_tree();

    tree.insert_file(
        "small.txt",
        &source_file("insert-inline-src", b"hello"),
        Stat::uninitialized(),
    )
    .unwrap();

    assert_eq!(tree.read_file("small.txt").unwrap(), b"hello");
}

#[test]
fn a_large_file_round_trips_through_the_repository() {
    let (_scratch, repo) = open_repo("insert-external");
    let mut tree = repo.empty_tree();
    let content = vec![7u8; 4096];

    tree.insert_file(
        "large.bin",
        &source_file("insert-external-src", &content),
        Stat::uninitialized(),
    )
    .unwrap();

    assert_eq!(tree.read_file("large.bin").unwrap(), content);
}

#[test]
fn inserted_bytes_round_trip_inline_and_through_the_repository() {
    let (_scratch, repo) = open_repo("insert-bytes");
    let mut tree = repo.empty_tree();
    let large_content = vec![7u8; 4096];

    tree.insert_bytes("small.txt", b"hello", Stat::uninitialized()).unwrap();
    tree.insert_bytes("large.bin", &large_content, Stat::uninitialized())
        .unwrap();

    assert_eq!(tree.read_file("small.txt").unwrap(), b"hello");
    assert_eq!(tree.read_file("large.bin").unwrap(), large_content);
}

#[test]
fn inserting_over_an_existing_file_replaces_it() {
    let (_scratch, repo) = open_repo("replace");
    let mut tree = repo.empty_tree();

    tree.insert_file(
        "file.txt",
        &source_file("replace-src-1", b"first"),
        Stat::uninitialized(),
    )
    .unwrap();
    tree.insert_file(
        "file.txt",
        &source_file("replace-src-2", b"second"),
        Stat::uninitialized(),
    )
    .unwrap();

    assert_eq!(tree.read_file("file.txt").unwrap(), b"second");
}

#[test]
fn a_clone_is_independent_of_the_original() {
    let (_scratch, repo) = open_repo("clone");
    let mut original = repo.empty_tree();
    original.insert_dir("dir", Stat::uninitialized()).unwrap();

    let mut copy = original.clone();
    copy.remove("dir").unwrap();

    assert!(original.contains("dir"));
    assert!(!copy.contains("dir"));
}

#[test]
fn import_dir_inserts_files_dirs_and_symlinks() {
    let source_dir = scratch_dir("import-source");
    create_dir_all(source_dir.path().join("sub")).unwrap();
    write(source_dir.path().join("sub/file.txt"), b"content").unwrap();
    symlink("file.txt", source_dir.path().join("sub/link")).unwrap();

    let (_scratch, repo) = open_repo("import-repo");
    let mut tree = repo.empty_tree();
    tree.insert_dir("target", Stat::uninitialized()).unwrap();

    let mut imported = tree
        .import_dir("target", source_dir.path(), &CancelToken::new(), &mut |_| {})
        .unwrap();
    imported.sort();

    assert_eq!(imported, vec![PathBuf::from("sub/file.txt"), PathBuf::from("sub/link")]);
    assert!(tree.contains("target/sub"));
    assert_eq!(tree.read_file("target/sub/file.txt").unwrap(), b"content");
    assert!(tree.contains("target/sub/link"));
}

#[test]
fn import_dir_merges_into_an_existing_directory_instead_of_replacing_it() {
    let first_source = scratch_dir("merge-first");
    create_dir_all(first_source.path().join("lib/modules/7.0")).unwrap();
    write(first_source.path().join("lib/modules/7.0/vmlinuz"), b"kernel").unwrap();

    let second_source = scratch_dir("merge-second");
    create_dir_all(second_source.path().join("lib")).unwrap();
    write(second_source.path().join("lib/libupac.so"), b"library").unwrap();

    let (_scratch, repo) = open_repo("merge-repo");
    let mut tree = repo.empty_tree();

    for source in [&first_source, &second_source] {
        tree.import_dir("", source.path(), &CancelToken::new(), &mut |_| {})
            .unwrap();
    }

    assert_eq!(tree.read_file("lib/modules/7.0/vmlinuz").unwrap(), b"kernel");
    assert_eq!(tree.read_file("lib/libupac.so").unwrap(), b"library");
}

#[test]
fn import_dir_replaces_a_file_that_stands_where_a_directory_is_imported() {
    let source_dir = scratch_dir("replace-leaf-source");
    create_dir_all(source_dir.path().join("conflict")).unwrap();
    write(source_dir.path().join("conflict/inner.txt"), b"inner").unwrap();

    let (_scratch, repo) = open_repo("replace-leaf-repo");
    let mut tree = repo.empty_tree();
    tree.insert_file(
        "conflict",
        &source_file("replace-leaf-file", b"was a file"),
        Stat::uninitialized(),
    )
    .unwrap();

    tree.import_dir("", source_dir.path(), &CancelToken::new(), &mut |_| {})
        .unwrap();

    assert_eq!(tree.read_file("conflict/inner.txt").unwrap(), b"inner");
}

#[test]
fn import_dir_stops_when_cancelled() {
    let source_dir = scratch_dir("cancel-source");
    write(source_dir.path().join("file.txt"), b"content").unwrap();

    let (_scratch, repo) = open_repo("cancel-repo");
    let mut tree = repo.empty_tree();
    let cancel = CancelToken::new();
    cancel.cancel();

    let result = tree.import_dir("", source_dir.path(), &cancel, &mut |_| {});

    assert!(matches!(result, Err(RepoError::Cancelled)));
}

#[test]
fn import_path_takes_a_single_file_with_its_content() {
    let source_dir = scratch_dir("import-path-source");
    write(source_dir.path().join("file.txt"), b"content").unwrap();

    let (_scratch, repo) = open_repo("import-path-repo");
    let mut tree = repo.empty_tree();

    tree.import_path("file.txt", &source_dir.path().join("file.txt"))
        .unwrap();

    assert_eq!(tree.read_file("file.txt").unwrap(), b"content");
}

#[test]
fn export_dir_writes_files_and_symlinks() {
    let source_dir = scratch_dir("export-source");
    create_dir_all(source_dir.path().join("sub")).unwrap();
    write(source_dir.path().join("sub/file.txt"), b"content").unwrap();
    symlink("file.txt", source_dir.path().join("sub/link")).unwrap();

    let (_scratch, repo) = open_repo("export-repo");
    let mut tree = repo.empty_tree();
    tree.import_dir("", source_dir.path(), &CancelToken::new(), &mut |_| {})
        .unwrap();

    let dest = scratch_dir("export-dest");
    tree.export_dir("", dest.path(), &CancelToken::new()).unwrap();

    assert_eq!(read(dest.path().join("sub/file.txt")).unwrap(), b"content");
    assert_eq!(
        read_link(dest.path().join("sub/link")).unwrap(),
        PathBuf::from("file.txt")
    );
}

#[test]
fn a_committed_tree_reopens_unchanged() {
    let (_scratch, repo) = open_repo("commit-round-trip");
    let mut tree = repo.empty_tree();
    tree.insert_file("file.txt", &source_file("commit-src", b"hello"), Stat::uninitialized())
        .unwrap();

    let before = tree.clone();
    let digest = tree.commit().unwrap();
    let after = repo.open_tree(&digest).unwrap();

    assert!(before.diff(&after).is_empty());
    assert_eq!(after.read_file("file.txt").unwrap(), b"hello");
}

#[test]
fn a_digest_round_trips_through_hex() {
    let (_scratch, repo) = open_repo("digest-hex");
    let digest = repo.empty_tree().commit().unwrap();

    assert_eq!(Digest::from_hex(&digest.to_hex()).unwrap(), digest);
    assert_eq!(digest.to_string(), digest.to_hex());
}

#[test]
fn an_invalid_digest_is_rejected() {
    assert!(matches!(
        Digest::from_hex("not-a-digest"),
        Err(RepoError::InvalidDigest)
    ));
}

#[test]
fn copy_tree_takes_only_the_requested_directory() {
    let (_scratch, repo) = open_repo("copy-tree");
    let mut tree = repo.empty_tree();
    tree.insert_dir("etc", Stat::uninitialized()).unwrap();
    tree.insert_dir("etc/sub", Stat::uninitialized()).unwrap();
    tree.insert_file(
        "etc/sub/conf",
        &source_file("copy-tree-conf", b"conf"),
        Stat::uninitialized(),
    )
    .unwrap();
    tree.insert_dir("bin", Stat::uninitialized()).unwrap();
    tree.insert_file(
        "bin/tool",
        &source_file("copy-tree-tool", b"tool"),
        Stat::uninitialized(),
    )
    .unwrap();

    let copy = tree.copy_tree("etc").unwrap();

    assert_eq!(copy.read_file("sub/conf").unwrap(), b"conf");
    assert!(!copy.contains("bin"));
    assert!(!copy.contains("etc"));
}

#[test]
fn a_copied_tree_has_no_dangling_leaves_and_commits() {
    let (_scratch, repo) = open_repo("copy-tree-commit");
    let mut tree = repo.empty_tree();
    tree.insert_dir("etc", Stat::uninitialized()).unwrap();
    tree.insert_file(
        "etc/conf",
        &source_file("copy-commit-conf", b"conf"),
        Stat::uninitialized(),
    )
    .unwrap();
    tree.insert_file(
        "other",
        &source_file("copy-commit-other", b"other"),
        Stat::uninitialized(),
    )
    .unwrap();

    let digest = tree.copy_tree("etc").unwrap().commit().unwrap();

    assert_eq!(repo.open_tree(&digest).unwrap().read_file("conf").unwrap(), b"conf");
}

#[test]
fn copying_a_missing_directory_is_not_found() {
    let (_scratch, repo) = open_repo("copy-tree-missing");
    let tree = repo.empty_tree();

    assert!(matches!(tree.copy_tree("etc"), Err(RepoError::NotFound)));
}
