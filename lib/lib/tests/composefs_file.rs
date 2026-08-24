// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs::{File, create_dir_all, write};
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use composefs::generic_tree::Stat;
use composefs::repository::{ImportContext, Repository, RepositoryConfig};
use composefs::tree::FileSystem;
use nix::fcntl::AT_FDCWD;
use tempfile::{Builder, TempDir};
use upac::composefs::error::RepoError;
use upac::composefs::file::FileHandle;
use upac::composefs::repository::ObjectID;
use upac_abi::hook::CancelToken;

fn scratch_dir(name: &str) -> TempDir {
    Builder::new().prefix(name).tempdir().unwrap()
}

fn empty_tree() -> FileSystem<ObjectID> {
    FileSystem::new(Stat::uninitialized())
}

fn open_repository(name: &str) -> (TempDir, Repository<ObjectID>) {
    let dir = scratch_dir(name);
    let (repository, _created) =
        Repository::init_path(AT_FDCWD, dir.path(), RepositoryConfig::default().set_insecure()).unwrap();

    (dir, repository)
}

fn source_file(dir_name: &str, content: &[u8]) -> File {
    let dir = scratch_dir(dir_name);
    let path = dir.path().join("source");
    write(&path, content).unwrap();

    File::open(&path).unwrap()
}

#[test]
fn insert_in_tree_then_stat_in_tree_finds_the_entry() {
    let mut tree = empty_tree();
    let handle = FileHandle::new("dir");

    handle.insert_in_tree(&mut tree, Stat::uninitialized()).unwrap();

    assert!(handle.stat_in_tree(&tree).is_ok());
}

#[test]
fn update_in_tree_changes_stat_without_dropping_children() {
    let mut tree = empty_tree();
    let dir = FileHandle::new("dir");
    dir.insert_in_tree(&mut tree, Stat::uninitialized()).unwrap();

    let child = FileHandle::new("dir/child");
    child.insert_in_tree(&mut tree, Stat::uninitialized()).unwrap();

    let mut new_stat = Stat::uninitialized();
    new_stat.st_mode = 0o755;
    dir.update_in_tree(&mut tree, new_stat).unwrap();

    assert_eq!(dir.stat_in_tree(&tree).unwrap().st_mode, 0o755);
    assert!(child.stat_in_tree(&tree).is_ok());
}

#[test]
fn rename_in_tree_moves_entry_and_children() {
    let mut tree = empty_tree();
    let mut dir = FileHandle::new("old");
    dir.insert_in_tree(&mut tree, Stat::uninitialized()).unwrap();

    let child = FileHandle::new("old/child");
    child.insert_in_tree(&mut tree, Stat::uninitialized()).unwrap();

    dir.rename_in_tree(&mut tree, "new").unwrap();

    assert!(FileHandle::new("old").stat_in_tree(&tree).is_err());
    assert!(FileHandle::new("new").stat_in_tree(&tree).is_ok());
    assert!(FileHandle::new("new/child").stat_in_tree(&tree).is_ok());
}

#[test]
fn remove_in_tree_drops_the_entry() {
    let mut tree = empty_tree();
    let handle = FileHandle::new("dir");
    handle.insert_in_tree(&mut tree, Stat::uninitialized()).unwrap();

    handle.remove_in_tree(&mut tree).unwrap();

    assert!(handle.stat_in_tree(&tree).is_err());
}

#[test]
fn symlink_in_tree_then_symlink_target_in_tree_round_trips() {
    let mut tree = empty_tree();
    let handle = FileHandle::new("link");

    handle
        .symlink_in_tree(&mut tree, "/usr/bin/target", Stat::uninitialized())
        .unwrap();

    assert_eq!(handle.symlink_target_in_tree(&tree).unwrap(), "/usr/bin/target");
}

#[test]
fn symlink_target_in_tree_fails_on_directory() {
    let mut tree = empty_tree();
    let handle = FileHandle::new("dir");
    handle.insert_in_tree(&mut tree, Stat::uninitialized()).unwrap();

    assert!(matches!(
        handle.symlink_target_in_tree(&tree),
        Err(RepoError::IsADirectory)
    ));
}

#[test]
fn hardlink_in_tree_shares_the_same_leaf() {
    let mut tree = empty_tree();
    let source = FileHandle::new("source");
    source
        .symlink_in_tree(&mut tree, "/target", Stat::uninitialized())
        .unwrap();

    let destination = FileHandle::new("destination");
    destination
        .hardlink_in_tree(&mut tree, &PathBuf::from("source"))
        .unwrap();

    assert_eq!(destination.symlink_target_in_tree(&tree).unwrap(), "/target");
}

#[test]
fn copy_from_tree_duplicates_the_leaf_into_a_different_tree() {
    let mut source_tree = empty_tree();
    FileHandle::new("source")
        .symlink_in_tree(&mut source_tree, "/target", Stat::uninitialized())
        .unwrap();

    let mut dest_tree = empty_tree();
    FileHandle::new("destination")
        .copy_from_tree(&mut dest_tree, &source_tree, &PathBuf::from("source"))
        .unwrap();

    assert_eq!(
        FileHandle::new("destination")
            .symlink_target_in_tree(&dest_tree)
            .unwrap(),
        "/target"
    );
    assert!(FileHandle::new("destination").stat_in_tree(&source_tree).is_err());
}

#[test]
fn list_in_tree_enumerates_children() {
    let mut tree = empty_tree();
    let dir = FileHandle::new("dir");
    dir.insert_in_tree(&mut tree, Stat::uninitialized()).unwrap();
    FileHandle::new("dir/a")
        .insert_in_tree(&mut tree, Stat::uninitialized())
        .unwrap();
    FileHandle::new("dir/b")
        .insert_in_tree(&mut tree, Stat::uninitialized())
        .unwrap();

    let names: Vec<String> = dir
        .list_in_tree(&tree)
        .unwrap()
        .map(|(name, _)| name.to_string_lossy().into_owned())
        .collect();

    assert_eq!(names, vec!["a".to_owned(), "b".to_owned()]);
}

#[test]
fn from_tree_succeeds_for_existing_path_and_fails_for_missing_path() {
    let mut tree = empty_tree();
    FileHandle::new("dir")
        .insert_in_tree(&mut tree, Stat::uninitialized())
        .unwrap();

    assert!(FileHandle::from_tree(&tree, "dir").is_ok());
    assert!(matches!(
        FileHandle::from_tree(&tree, "missing"),
        Err(RepoError::NotFound)
    ));
}

#[test]
fn insert_file_inline_then_read_file_round_trips() {
    let (_scratch, repository) = open_repository("insert-inline");
    let mut tree = empty_tree();
    let mut ctx = ImportContext::default();
    let handle = FileHandle::new("small.txt");

    handle
        .insert_file(
            &repository,
            &mut tree,
            &source_file("insert-inline-src", b"hello"),
            Stat::uninitialized(),
            &mut ctx,
        )
        .unwrap();

    assert_eq!(handle.read_file(&repository, &tree).unwrap(), b"hello");
}

#[test]
fn insert_file_external_then_read_file_round_trips() {
    let (_scratch, repository) = open_repository("insert-external");
    let mut tree = empty_tree();
    let mut ctx = ImportContext::default();
    let handle = FileHandle::new("large.bin");

    let content = vec![7u8; 4096];
    handle
        .insert_file(
            &repository,
            &mut tree,
            &source_file("insert-external-src", &content),
            Stat::uninitialized(),
            &mut ctx,
        )
        .unwrap();

    assert_eq!(handle.read_file(&repository, &tree).unwrap(), content);
}

#[test]
fn replace_file_overwrites_previous_content() {
    let (_scratch, repository) = open_repository("replace");
    let mut tree = empty_tree();
    let mut ctx = ImportContext::default();
    let handle = FileHandle::new("file.txt");

    handle
        .insert_file(
            &repository,
            &mut tree,
            &source_file("replace-src-1", b"first"),
            Stat::uninitialized(),
            &mut ctx,
        )
        .unwrap();
    handle
        .replace_file(
            &repository,
            &mut tree,
            &source_file("replace-src-2", b"second"),
            Stat::uninitialized(),
            &mut ctx,
        )
        .unwrap();

    assert_eq!(handle.read_file(&repository, &tree).unwrap(), b"second");
}

#[test]
fn import_directory_inserts_files_dirs_and_symlinks() {
    let source_dir = scratch_dir("import-source");
    create_dir_all(source_dir.path().join("sub")).unwrap();
    write(source_dir.path().join("sub/file.txt"), b"content").unwrap();
    symlink("file.txt", source_dir.path().join("sub/link")).unwrap();

    let (_scratch, repository) = open_repository("import-repo");
    let mut tree = empty_tree();
    let mut ctx = ImportContext::default();
    let handle = FileHandle::new("target");
    handle.insert_in_tree(&mut tree, Stat::uninitialized()).unwrap();

    let cancel = CancelToken::new();
    let mut imported = handle
        .import_directory(&repository, &mut tree, source_dir.path(), &mut ctx, &cancel)
        .unwrap();
    imported.sort();

    assert_eq!(imported, vec![PathBuf::from("sub/file.txt"), PathBuf::from("sub/link")]);
    assert!(FileHandle::new("target/sub").stat_in_tree(&tree).is_ok());
    assert_eq!(
        FileHandle::new("target/sub/file.txt")
            .read_file(&repository, &tree)
            .unwrap(),
        b"content"
    );
    assert_eq!(
        FileHandle::new("target/sub/link")
            .symlink_target_in_tree(&tree)
            .unwrap(),
        "file.txt"
    );
}
