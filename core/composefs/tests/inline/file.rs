// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::PathBuf;

use composefs::generic_tree::Stat;
use composefs::tree::FileSystem;

use super::{FileHandle, ObjectID};

fn empty_tree() -> FileSystem<ObjectID> {
    FileSystem::new(Stat::uninitialized())
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
