// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs::{File, write};
use std::io::Read;

use composefs::erofs::reader::erofs_to_filesystem;
use composefs::fsverity::{FsVerityHashValue, Sha256HashValue};
use composefs::generic_tree::Stat;
use composefs::repository::{ImportContext, Repository, RepositoryConfig};
use composefs::tree::FileSystem;
use nix::fcntl::AT_FDCWD;
use tempfile::{Builder, TempDir};
use upac::composefs::diff::TreeDiff;
use upac::composefs::file::FileHandle;
use upac::composefs::repository::{ObjectID, commit_tree};

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

fn open_image_tree(repository: &Repository<ObjectID>, digest: &Sha256HashValue) -> FileSystem<ObjectID> {
    let (image, _enable_verity) = repository.open_image(&digest.to_hex()).unwrap();

    let mut data = Vec::new();
    File::from(image).read_to_end(&mut data).unwrap();

    erofs_to_filesystem(&data).unwrap()
}

#[test]
fn commit_tree_round_trips_through_the_repository() {
    let (_scratch, repository) = open_repository("commit-round-trip");
    let mut tree = empty_tree();
    let mut ctx = ImportContext::default();

    let source_dir = scratch_dir("commit-round-trip-src");
    let source_path = source_dir.path().join("source");
    write(&source_path, b"hello").unwrap();

    FileHandle::new("file.txt")
        .insert_file(
            &repository,
            &mut tree,
            &File::open(&source_path).unwrap(),
            Stat::uninitialized(),
            &mut ctx,
        )
        .unwrap();

    let before = tree.clone();
    let digest = commit_tree(&repository, tree).unwrap();
    let after = open_image_tree(&repository, &digest);

    assert!(TreeDiff::run(&before, &after).is_empty());
}
