// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::Path;

use composefs::generic_tree::Stat;
use composefs::repository::{Repository, RepositoryConfig};

use nix::fcntl::AT_FDCWD;

use tempfile::TempDir;

use super::super::{ObjectID, Repo};

fn open_repo() -> (TempDir, Repo) {
    let dir = TempDir::new().unwrap();
    Repository::<ObjectID>::init_path(AT_FDCWD, dir.path(), RepositoryConfig::default().set_insecure()).unwrap();
    let repo = Repo::open(dir.path()).unwrap();

    (dir, repo)
}

#[test]
fn setting_a_directory_stat_keeps_its_children() {
    let (_scratch, repo) = open_repo();
    let mut tree = repo.empty_tree();
    tree.insert_dir("dir", Stat::uninitialized()).unwrap();
    tree.insert_dir("dir/child", Stat::uninitialized()).unwrap();

    let mut new_stat = Stat::uninitialized();
    new_stat.st_mode = 0o755;
    tree.set_dir_stat(Path::new("dir"), new_stat).unwrap();

    assert_eq!(tree.stat("dir").unwrap().st_mode, 0o755);
    assert!(tree.contains("dir/child"));
}

#[test]
fn copying_a_leaf_duplicates_it_into_another_tree() {
    let (_scratch, repo) = open_repo();
    let mut source = repo.empty_tree();
    source
        .insert_symlink("source", "/target", Stat::uninitialized())
        .unwrap();

    let mut destination = repo.empty_tree();
    destination
        .copy_leaf(Path::new("destination"), &source, Path::new("source"))
        .unwrap();

    assert!(destination.contains("destination"));
    assert!(!source.contains("destination"));
}
