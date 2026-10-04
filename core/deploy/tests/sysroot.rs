// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, create_dir_all, write};
use std::path::Path;

use composefs::generic_tree::Stat;
use composefs::repository::{Repository, RepositoryConfig};

use nix::fcntl::AT_FDCWD;

use tempfile::TempDir;

use upac_types::transaction::{Transaction, TransactionKind};

use upac_composefs::{Digest, ObjectID};

use upac_database::MemoryDatabase;
use upac_database::layout::database::DATABASE_PATH;
use upac_database::transaction::TransactionStoreMut;

use upac_deploy::Sysroot;
use upac_deploy::deployment::config::ConfigDeploy;
use upac_deploy::deployment::{Deployment, PrefixDeploy};
use upac_deploy::error::{PrefixCreateError, PrefixMetaError, PrefixReadError, SysrootError};
use upac_deploy::layout::deployment::{CONFIG_DIR_NAME, DEPLOYS_DIR, LIVE_ETC_UPPER_DIR_NAME, REPO_DIR};

fn scratch_root() -> TempDir {
    let scratch = TempDir::new().unwrap();

    create_dir_all(scratch.path().join(DEPLOYS_DIR)).unwrap();
    Repository::<ObjectID>::init_path(
        AT_FDCWD,
        scratch.path().join(REPO_DIR),
        RepositoryConfig::default().set_insecure(),
    )
    .unwrap();

    scratch
}

fn digest(seed: u8) -> Digest {
    Digest::from_hex(&format!("{seed:02x}").repeat(32)).unwrap()
}

fn transaction() -> Transaction {
    Transaction::new(None, TransactionKind::Install, "install foo".to_owned(), None)
}

fn commit_prefix_tree(sysroot: &Sysroot, scratch: &Path, transaction: Option<&Transaction>) -> Digest {
    let mut database = MemoryDatabase::new_in_memory().unwrap();
    if let Some(transaction) = transaction {
        database.set_transaction(transaction).unwrap();
    }

    let database_path = scratch.join("packages.redb");
    write(&database_path, database.into_bytes().unwrap()).unwrap();

    let mut tree = sysroot.repo().empty_tree();
    tree.insert_dir("share", Stat::uninitialized()).unwrap();
    tree.insert_dir("share/upac", Stat::uninitialized()).unwrap();
    tree.insert_file(
        DATABASE_PATH,
        &File::open(&database_path).unwrap(),
        Stat::uninitialized(),
    )
    .unwrap();

    tree.commit().unwrap()
}

fn new_prefix(sysroot: &Sysroot, scratch: &Path) -> PrefixDeploy {
    let transaction = transaction();
    let prefix_digest = commit_prefix_tree(sysroot, scratch, Some(&transaction));
    let config = ConfigDeploy::new(digest(11), "install".to_owned(), None);

    PrefixDeploy::new(prefix_digest, transaction, config)
}

#[test]
fn opening_fails_when_the_deploys_dir_is_missing() {
    let scratch = TempDir::new().unwrap();
    create_dir_all(scratch.path().join(REPO_DIR)).unwrap();

    let result = Sysroot::open(scratch.path());

    assert_eq!(result.err(), Some(SysrootError::DeploysDirNotFound));
}

#[test]
fn opening_fails_when_the_repository_dir_is_missing() {
    let scratch = TempDir::new().unwrap();
    create_dir_all(scratch.path().join(DEPLOYS_DIR)).unwrap();

    let result = Sysroot::open(scratch.path());

    assert_eq!(result.err(), Some(SysrootError::RepoDirNotFound));
}

#[test]
fn a_created_prefix_reads_back_with_its_transaction_and_state() {
    let scratch = scratch_root();
    let sysroot = Sysroot::open(scratch.path()).unwrap();
    let prefix = new_prefix(&sysroot, scratch.path());

    sysroot.create_prefix(&prefix).unwrap();

    assert_eq!(sysroot.prefix(prefix.digest()), Ok(prefix));
}

#[test]
fn creating_an_existing_prefix_fails() {
    let scratch = scratch_root();
    let sysroot = Sysroot::open(scratch.path()).unwrap();
    let prefix = new_prefix(&sysroot, scratch.path());

    sysroot.create_prefix(&prefix).unwrap();

    assert_eq!(
        sysroot.create_prefix(&prefix),
        Err(PrefixCreateError::AlreadyExists(prefix.digest().clone()))
    );
}

#[test]
fn saved_changes_are_read_back() {
    let scratch = scratch_root();
    let sysroot = Sysroot::open(scratch.path()).unwrap();
    let mut prefix = new_prefix(&sysroot, scratch.path());
    sysroot.create_prefix(&prefix).unwrap();

    prefix.set_pinned(true);
    prefix.add_config(ConfigDeploy::new(digest(12), "commit".to_owned(), None));
    sysroot.save_prefix(&prefix).unwrap();

    assert_eq!(sysroot.prefix(prefix.digest()), Ok(prefix));
}

#[test]
fn identical_content_from_two_transactions_gives_two_prefixes() {
    let scratch = scratch_root();
    let sysroot = Sysroot::open(scratch.path()).unwrap();

    let first = new_prefix(&sysroot, scratch.path());
    let second = new_prefix(&sysroot, scratch.path());
    sysroot.create_prefix(&first).unwrap();
    sysroot.create_prefix(&second).unwrap();

    assert_ne!(first.digest(), second.digest());
    assert_eq!(sysroot.prefixes().unwrap().len(), 2);
}

#[test]
fn removing_a_prefix_deletes_it_and_repeating_it_is_harmless() {
    let scratch = scratch_root();
    let sysroot = Sysroot::open(scratch.path()).unwrap();
    let prefix = new_prefix(&sysroot, scratch.path());
    sysroot.create_prefix(&prefix).unwrap();

    sysroot.remove_prefix(prefix.digest()).unwrap();
    sysroot.remove_prefix(prefix.digest()).unwrap();

    assert!(sysroot.prefixes().unwrap().is_empty());
}

#[test]
fn a_tree_without_a_transaction_cannot_be_read_as_a_prefix() {
    let scratch = scratch_root();
    let sysroot = Sysroot::open(scratch.path()).unwrap();
    let tree_digest = commit_prefix_tree(&sysroot, scratch.path(), None);

    assert_eq!(sysroot.prefix(&tree_digest), Err(PrefixReadError::TransactionMissing));
}

#[test]
fn the_live_etc_upper_dir_lives_inside_the_prefix_dir() {
    let scratch = scratch_root();
    let sysroot = Sysroot::open(scratch.path()).unwrap();

    let expected = scratch
        .path()
        .join(DEPLOYS_DIR)
        .join(digest(1).to_hex())
        .join(CONFIG_DIR_NAME)
        .join(LIVE_ETC_UPPER_DIR_NAME);

    assert_eq!(sysroot.live_etc_upper_dir(&digest(1)), expected);
}

#[test]
fn the_next_prefix_reads_back_what_was_set() {
    let scratch = scratch_root();
    let sysroot = Sysroot::open(scratch.path()).unwrap();
    let prefix = new_prefix(&sysroot, scratch.path());
    sysroot.create_prefix(&prefix).unwrap();

    sysroot.set_next_prefix(&prefix).unwrap();

    assert_eq!(sysroot.next_prefix(), Ok(prefix));
}

#[test]
fn the_next_prefix_is_missing_until_it_is_set() {
    let scratch = scratch_root();
    let sysroot = Sysroot::open(scratch.path()).unwrap();

    assert_eq!(
        sysroot.next_prefix(),
        Err(PrefixReadError::Meta(PrefixMetaError::NotFound))
    );
}

#[test]
fn the_next_pointer_does_not_count_as_a_prefix() {
    let scratch = scratch_root();
    let sysroot = Sysroot::open(scratch.path()).unwrap();
    let prefix = new_prefix(&sysroot, scratch.path());
    sysroot.create_prefix(&prefix).unwrap();
    sysroot.set_next_prefix(&prefix).unwrap();

    assert_eq!(sysroot.prefixes().unwrap().len(), 1);
}

#[test]
fn a_created_prefix_has_the_etc_dir_setup_root_expects() {
    let scratch = scratch_root();
    let sysroot = Sysroot::open(scratch.path()).unwrap();
    let prefix = new_prefix(&sysroot, scratch.path());

    sysroot.create_prefix(&prefix).unwrap();

    let etc_dir = scratch
        .path()
        .join(DEPLOYS_DIR)
        .join(prefix.digest().to_hex())
        .join(CONFIG_DIR_NAME);
    assert!(etc_dir.is_dir());
}
