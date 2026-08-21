// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::env::temp_dir;
use std::fs::{File, create_dir_all, remove_dir_all, write};
use std::path::PathBuf;
use std::process::id;

use composefs::generic_tree::Stat;
use composefs::repository::{ImportContext, Repository, RepositoryConfig};
use composefs::tree::FileSystem;
use nix::fcntl::AT_FDCWD;
use upac::composefs::file::FileHandle;
use upac::composefs::repository::ObjectID;
use upac::config::merge::merge_config;

fn scratch_dir(name: &str) -> PathBuf {
    let dir = temp_dir().join(format!("upac-test-config-merge-{}-{name}", id()));
    let _ = remove_dir_all(&dir);
    create_dir_all(&dir).unwrap();

    dir
}

fn empty_tree() -> FileSystem<ObjectID> {
    FileSystem::new(Stat::uninitialized())
}

fn open_repository(name: &str) -> Repository<ObjectID> {
    let dir = scratch_dir(name);
    let (repository, _created) =
        Repository::init_path(AT_FDCWD, &dir, RepositoryConfig::default().set_insecure()).unwrap();

    repository
}

fn source_file(dir_name: &str, content: &[u8]) -> File {
    let dir = scratch_dir(dir_name);
    let path = dir.join("source");
    write(&path, content).unwrap();

    File::open(&path).unwrap()
}

fn insert(
    repository: &Repository<ObjectID>, tree: &mut FileSystem<ObjectID>, ctx: &mut ImportContext, label: &str,
    path: &str, content: &[u8],
) {
    FileHandle::new(path)
        .insert_file(
            repository,
            tree,
            &source_file(label, content),
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

#[test]
fn untouched_file_keeps_the_new_package_default() {
    let repository = open_repository("untouched");
    let mut ctx = ImportContext::default();

    let mut base = empty_tree();
    insert(&repository, &mut base, &mut ctx, "untouched-base", "conf", b"base");
    let live = base.clone();

    let mut new = empty_tree();
    insert(&repository, &mut new, &mut ctx, "untouched-new", "conf", b"new");

    let result = merge_config(&base, &new, &live).unwrap();

    assert_eq!(read(&repository, &result.tree, "conf"), b"new");
    assert!(result.conflicts.is_empty());
}

#[test]
fn user_only_edit_is_kept_when_package_did_not_change_the_file() {
    let repository = open_repository("user-only-edit");
    let mut ctx = ImportContext::default();

    let mut base = empty_tree();
    insert(&repository, &mut base, &mut ctx, "user-only-edit-base", "conf", b"base");
    let new = base.clone();

    let mut live = empty_tree();
    insert(
        &repository,
        &mut live,
        &mut ctx,
        "user-only-edit-live",
        "conf",
        b"user-edit",
    );

    let result = merge_config(&base, &new, &live).unwrap();

    assert_eq!(read(&repository, &result.tree, "conf"), b"user-edit");
    assert!(result.conflicts.is_empty());
}

#[test]
fn conflicting_edit_keeps_the_user_version_and_writes_upac_new_sidecar() {
    let repository = open_repository("conflict-edit");
    let mut ctx = ImportContext::default();

    let mut base = empty_tree();
    insert(&repository, &mut base, &mut ctx, "conflict-edit-base", "conf", b"base");

    let mut new = empty_tree();
    insert(
        &repository,
        &mut new,
        &mut ctx,
        "conflict-edit-new",
        "conf",
        b"package-new",
    );

    let mut live = empty_tree();
    insert(
        &repository,
        &mut live,
        &mut ctx,
        "conflict-edit-live",
        "conf",
        b"user-edit",
    );

    let result = merge_config(&base, &new, &live).unwrap();

    assert_eq!(read(&repository, &result.tree, "conf"), b"user-edit");
    assert_eq!(read(&repository, &result.tree, "conf.upac-new"), b"package-new");
    assert_eq!(result.conflicts, vec!["conf".to_owned()]);
}

#[test]
fn user_deletion_is_carried_over_when_the_package_did_not_change_the_file() {
    let repository = open_repository("user-deletion");
    let mut ctx = ImportContext::default();

    let mut base = empty_tree();
    insert(&repository, &mut base, &mut ctx, "user-deletion-base", "conf", b"base");
    let new = base.clone();
    let live = empty_tree();

    let result = merge_config(&base, &new, &live).unwrap();

    assert!(!exists(&result.tree, "conf"));
    assert!(result.conflicts.is_empty());
}

#[test]
fn user_deletion_conflicts_when_the_package_also_changed_the_file() {
    let repository = open_repository("user-deletion-conflict");
    let mut ctx = ImportContext::default();

    let mut base = empty_tree();
    insert(
        &repository,
        &mut base,
        &mut ctx,
        "user-deletion-conflict-base",
        "conf",
        b"base",
    );

    let mut new = empty_tree();
    insert(
        &repository,
        &mut new,
        &mut ctx,
        "user-deletion-conflict-new",
        "conf",
        b"package-new",
    );

    let live = empty_tree();

    let result = merge_config(&base, &new, &live).unwrap();

    assert_eq!(read(&repository, &result.tree, "conf"), b"package-new");
    assert!(!exists(&result.tree, "conf.upac-new"));
    assert_eq!(result.conflicts, vec!["conf".to_owned()]);
}

#[test]
fn brand_new_user_file_survives_the_merge() {
    let repository = open_repository("brand-new-user-file");
    let mut ctx = ImportContext::default();

    let base = empty_tree();
    let new = empty_tree();

    let mut live = empty_tree();
    insert(
        &repository,
        &mut live,
        &mut ctx,
        "brand-new-user-file-live",
        "conf",
        b"user-only",
    );

    let result = merge_config(&base, &new, &live).unwrap();

    assert_eq!(read(&repository, &result.tree, "conf"), b"user-only");
    assert!(result.conflicts.is_empty());
}
