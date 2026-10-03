// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, create_dir_all, write};
use std::path::Path;

use composefs::generic_tree::Stat;
use composefs::repository::{Repository, RepositoryConfig};

use nix::fcntl::AT_FDCWD;
use nix::sys::stat::{Mode, SFlag, mknod};

use tempfile::{Builder, TempDir};

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

fn source_file(label: &str, content: &[u8]) -> File {
    let dir = scratch_dir(label);
    let path = dir.path().join("source");
    write(&path, content).unwrap();

    File::open(&path).unwrap()
}

fn insert(tree: &mut Tree, label: &str, path: &str, content: &[u8]) {
    tree.insert_file(path, &source_file(label, content), Stat::uninitialized())
        .unwrap();
}

fn read(tree: &Tree, path: &str) -> Vec<u8> {
    tree.read_file(path).unwrap()
}

fn exists(tree: &Tree, path: &str) -> bool {
    tree.contains(path)
}

fn write_whiteout(path: &Path) {
    mknod(path, SFlag::S_IFCHR, Mode::from_bits_truncate(0o644), 0).unwrap();
}

#[test]
fn untouched_base_entry_survives_when_upper_does_not_touch_it() {
    let (_scratch, repo) = open_repo("untouched-repo");

    let mut tree = repo.empty_tree();
    insert(&mut tree, "untouched-base", "keep.txt", b"base content");

    let upper = scratch_dir("untouched-upper");
    write(upper.path().join("unrelated.txt"), b"something else").unwrap();

    tree.apply_overlay_upper(upper.path()).unwrap();

    assert_eq!(read(&tree, "keep.txt"), b"base content");
}

#[test]
fn upper_file_overrides_base_file() {
    let (_scratch, repo) = open_repo("override-repo");

    let mut tree = repo.empty_tree();
    insert(&mut tree, "override-base", "conf", b"base content");

    let upper = scratch_dir("override-upper");
    write(upper.path().join("conf"), b"user edit").unwrap();

    tree.apply_overlay_upper(upper.path()).unwrap();

    assert_eq!(read(&tree, "conf"), b"user edit");
}

#[test]
fn whiteout_in_upper_removes_base_entry() {
    let (_scratch, repo) = open_repo("whiteout-repo");

    let mut tree = repo.empty_tree();
    insert(&mut tree, "whiteout-base", "gone.txt", b"base content");

    let upper = scratch_dir("whiteout-upper");
    write_whiteout(&upper.path().join("gone.txt"));

    tree.apply_overlay_upper(upper.path()).unwrap();

    assert!(!exists(&tree, "gone.txt"));
}

#[test]
fn nested_directory_merges_without_opaque() {
    let (_scratch, repo) = open_repo("nested-repo");

    let mut tree = repo.empty_tree();
    tree.insert_dir("dir", Stat::uninitialized()).unwrap();
    insert(&mut tree, "nested-base-a", "dir/a.txt", b"a content");
    insert(&mut tree, "nested-base-b", "dir/b.txt", b"old b content");

    let upper = scratch_dir("nested-upper");
    create_dir_all(upper.path().join("dir")).unwrap();
    write(upper.path().join("dir/b.txt"), b"new b content").unwrap();

    tree.apply_overlay_upper(upper.path()).unwrap();

    assert_eq!(read(&tree, "dir/a.txt"), b"a content");
    assert_eq!(read(&tree, "dir/b.txt"), b"new b content");
}

#[test]
fn tree_overlay_leaves_untouched_base_entries_alone() {
    let (_scratch, repo) = open_repo("tree-untouched-repo");

    let mut base = repo.empty_tree();
    insert(&mut base, "tree-untouched-base", "keep.txt", b"base content");

    let overlay = repo.empty_tree();

    base.overlay(&overlay).unwrap();

    assert_eq!(read(&base, "keep.txt"), b"base content");
}

#[test]
fn tree_overlay_file_overrides_base_file() {
    let (_scratch, repo) = open_repo("tree-override-repo");

    let mut base = repo.empty_tree();
    insert(&mut base, "tree-override-base", "conf", b"base content");

    let mut overlay = repo.empty_tree();
    insert(&mut overlay, "tree-override-overlay", "conf", b"overlay content");

    base.overlay(&overlay).unwrap();

    assert_eq!(read(&base, "conf"), b"overlay content");
}

#[test]
fn tree_overlay_adds_brand_new_path() {
    let (_scratch, repo) = open_repo("tree-new-repo");

    let mut base = repo.empty_tree();
    insert(&mut base, "tree-new-base", "old.txt", b"old content");

    let mut overlay = repo.empty_tree();
    insert(&mut overlay, "tree-new-overlay", "new.txt", b"new content");

    base.overlay(&overlay).unwrap();

    assert_eq!(read(&base, "old.txt"), b"old content");
    assert_eq!(read(&base, "new.txt"), b"new content");
}

#[test]
fn tree_overlay_merges_nested_directory_without_removing_siblings() {
    let (_scratch, repo) = open_repo("tree-nested-repo");

    let mut base = repo.empty_tree();
    base.insert_dir("dir", Stat::uninitialized()).unwrap();
    insert(&mut base, "tree-nested-base-a", "dir/a.txt", b"a content");
    insert(&mut base, "tree-nested-base-b", "dir/b.txt", b"old b content");

    let mut overlay = repo.empty_tree();
    overlay.insert_dir("dir", Stat::uninitialized()).unwrap();
    insert(&mut overlay, "tree-nested-overlay-b", "dir/b.txt", b"new b content");

    base.overlay(&overlay).unwrap();

    assert_eq!(read(&base, "dir/a.txt"), b"a content");
    assert_eq!(read(&base, "dir/b.txt"), b"new b content");
}

#[test]
#[ignore = "trusted.overlay.opaque requires CAP_SYS_ADMIN (root) to set via setxattr"]
fn opaque_directory_drops_base_subtree_entirely() {
    let (_scratch, repo) = open_repo("opaque-repo");

    let mut tree = repo.empty_tree();
    tree.insert_dir("dir", Stat::uninitialized()).unwrap();
    insert(&mut tree, "opaque-base-old", "dir/old.txt", b"old content");

    let upper = scratch_dir("opaque-upper");
    let upper_dir = upper.path().join("dir");
    create_dir_all(&upper_dir).unwrap();
    xattr::set(&upper_dir, "trusted.overlay.opaque", b"y").unwrap();
    write(upper_dir.join("new.txt"), b"new content").unwrap();

    tree.apply_overlay_upper(upper.path()).unwrap();

    assert!(!exists(&tree, "dir/old.txt"));
    assert_eq!(read(&tree, "dir/new.txt"), b"new content");
}
