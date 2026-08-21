// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs::{File, create_dir_all, write};

use composefs::generic_tree::Stat;
use composefs::repository::{ImportContext, Repository, RepositoryConfig};
use composefs::tree::FileSystem;
use nix::fcntl::AT_FDCWD;
use nix::sys::stat::{Mode, SFlag, mknod};
use tempfile::{Builder, TempDir};
use upac::composefs::file::FileHandle;
use upac::composefs::overlay::apply_overlay_upper;
use upac::composefs::repository::ObjectID;

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

fn insert(
    repository: &Repository<ObjectID>, tree: &mut FileSystem<ObjectID>, ctx: &mut ImportContext, label: &str,
    path: &str, content: &[u8],
) {
    let dir = scratch_dir(label);
    let source_path = dir.path().join("source");
    write(&source_path, content).unwrap();

    FileHandle::new(path)
        .insert_file(
            repository,
            tree,
            &File::open(&source_path).unwrap(),
            Stat::uninitialized(),
            ctx,
        )
        .unwrap();
}

fn read(repository: &Repository<ObjectID>, tree: &FileSystem<ObjectID>, path: &str) -> Vec<u8> {
    FileHandle::new(path).read_file(repository, tree).unwrap()
}

fn exists(tree: &FileSystem<ObjectID>, path: &str) -> bool {
    FileHandle::new(path).stat_in_tree(tree).is_ok()
}

fn write_whiteout(path: &std::path::Path) {
    mknod(path, SFlag::S_IFCHR, Mode::from_bits_truncate(0o644), 0).unwrap();
}

#[test]
fn untouched_base_entry_survives_when_upper_does_not_touch_it() {
    let (_scratch, repository) = open_repository("untouched-repo");
    let mut ctx = ImportContext::default();

    let mut tree = empty_tree();
    insert(
        &repository,
        &mut tree,
        &mut ctx,
        "untouched-base",
        "keep.txt",
        b"base content",
    );

    let upper = scratch_dir("untouched-upper");
    write(upper.path().join("unrelated.txt"), b"something else").unwrap();

    apply_overlay_upper(&repository, &mut tree, upper.path(), &mut ctx).unwrap();

    assert_eq!(read(&repository, &tree, "keep.txt"), b"base content");
}

#[test]
fn upper_file_overrides_base_file() {
    let (_scratch, repository) = open_repository("override-repo");
    let mut ctx = ImportContext::default();

    let mut tree = empty_tree();
    insert(
        &repository,
        &mut tree,
        &mut ctx,
        "override-base",
        "conf",
        b"base content",
    );

    let upper = scratch_dir("override-upper");
    write(upper.path().join("conf"), b"user edit").unwrap();

    apply_overlay_upper(&repository, &mut tree, upper.path(), &mut ctx).unwrap();

    assert_eq!(read(&repository, &tree, "conf"), b"user edit");
}

#[test]
fn whiteout_in_upper_removes_base_entry() {
    let (_scratch, repository) = open_repository("whiteout-repo");
    let mut ctx = ImportContext::default();

    let mut tree = empty_tree();
    insert(
        &repository,
        &mut tree,
        &mut ctx,
        "whiteout-base",
        "gone.txt",
        b"base content",
    );

    let upper = scratch_dir("whiteout-upper");
    write_whiteout(&upper.path().join("gone.txt"));

    apply_overlay_upper(&repository, &mut tree, upper.path(), &mut ctx).unwrap();

    assert!(!exists(&tree, "gone.txt"));
}

#[test]
fn nested_directory_merges_without_opaque() {
    let (_scratch, repository) = open_repository("nested-repo");
    let mut ctx = ImportContext::default();

    let mut tree = empty_tree();
    FileHandle::new("dir")
        .insert_in_tree(&mut tree, Stat::uninitialized())
        .unwrap();
    insert(
        &repository,
        &mut tree,
        &mut ctx,
        "nested-base-a",
        "dir/a.txt",
        b"a content",
    );
    insert(
        &repository,
        &mut tree,
        &mut ctx,
        "nested-base-b",
        "dir/b.txt",
        b"old b content",
    );

    let upper = scratch_dir("nested-upper");
    create_dir_all(upper.path().join("dir")).unwrap();
    write(upper.path().join("dir/b.txt"), b"new b content").unwrap();

    apply_overlay_upper(&repository, &mut tree, upper.path(), &mut ctx).unwrap();

    assert_eq!(read(&repository, &tree, "dir/a.txt"), b"a content");
    assert_eq!(read(&repository, &tree, "dir/b.txt"), b"new b content");
}

#[test]
#[ignore = "trusted.overlay.opaque requires CAP_SYS_ADMIN (root) to set via setxattr"]
fn opaque_directory_drops_base_subtree_entirely() {
    let (_scratch, repository) = open_repository("opaque-repo");
    let mut ctx = ImportContext::default();

    let mut tree = empty_tree();
    FileHandle::new("dir")
        .insert_in_tree(&mut tree, Stat::uninitialized())
        .unwrap();
    insert(
        &repository,
        &mut tree,
        &mut ctx,
        "opaque-base-old",
        "dir/old.txt",
        b"old content",
    );

    let upper = scratch_dir("opaque-upper");
    let upper_dir = upper.path().join("dir");
    create_dir_all(&upper_dir).unwrap();
    xattr::set(&upper_dir, "trusted.overlay.opaque", b"y").unwrap();
    write(upper_dir.join("new.txt"), b"new content").unwrap();

    apply_overlay_upper(&repository, &mut tree, upper.path(), &mut ctx).unwrap();

    assert!(!exists(&tree, "dir/old.txt"));
    assert_eq!(read(&repository, &tree, "dir/new.txt"), b"new content");
}
