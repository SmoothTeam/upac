// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{Permissions, create_dir, set_permissions, write};
use std::os::unix::fs::{PermissionsExt, symlink};

use composefs::generic_tree::LeafContent;
use composefs::repository::{Repository, RepositoryConfig};

use nix::fcntl::AT_FDCWD;

use tempfile::{Builder, TempDir};

use upac_types::CancelToken;

use upac_composefs::error::RepoError;
use upac_composefs::tree::Tree;
use upac_composefs::{ObjectID, Repo};

fn open_repo(name: &str) -> (TempDir, Repo) {
    let scratch = Builder::new().prefix(name).tempdir().unwrap();
    Repository::<ObjectID>::init_path(AT_FDCWD, scratch.path(), RepositoryConfig::default().set_insecure()).unwrap();
    let repo = Repo::open(scratch.path()).unwrap();

    (scratch, repo)
}

#[test]
fn files_symlinks_and_empty_directories_come_from_the_host() {
    let (_scratch, repo) = open_repo("host-repo");
    let source = Builder::new().prefix("host-source").tempdir().unwrap();
    write(source.path().join("file"), b"content").unwrap();
    symlink("file", source.path().join("link")).unwrap();
    create_dir(source.path().join("empty")).unwrap();
    set_permissions(source.path().join("empty"), Permissions::from_mode(0o750)).unwrap();

    let tree = Tree::from_host(&repo, source.path(), &CancelToken::new(), &mut |_| {}).unwrap();

    assert!(
        matches!(tree.get("file").unwrap(), LeafContent::Regular(stored) if repo.read(&stored).unwrap() == b"content")
    );
    assert!(matches!(tree.get("link").unwrap(), LeafContent::Symlink(target) if &*target == "file"));
    assert_eq!(tree.get_metadata("empty").unwrap().st_mode & 0o7777, 0o750);
}

#[test]
fn reading_the_host_stops_when_cancelled() {
    let (_scratch, repo) = open_repo("cancel-repo");
    let source = Builder::new().prefix("cancel-source").tempdir().unwrap();
    write(source.path().join("file"), b"content").unwrap();
    let cancel = CancelToken::new();
    cancel.cancel();

    let result = Tree::from_host(&repo, source.path(), &cancel, &mut |_| {});

    assert!(matches!(result, Err(RepoError::Cancelled)));
}
