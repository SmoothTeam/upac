// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsStr;

use upac_composefs::tree::builder::TreeBuilder;

use upac_types::transaction::{Transaction, TransactionKind};

use upac_database::package::PackageStore;
use upac_database::transaction::TransactionStore;

fn builder_with_log() -> TreeBuilder {
    let mut builder = TreeBuilder::new();
    PackageStore::write(&mut builder).init().unwrap();
    TransactionStore::write(&mut builder).init().unwrap();

    builder
}

fn transaction(timestamp: u128, kind: TransactionKind, subject: &str, message: Option<&str>) -> Transaction {
    Transaction {
        timestamp,
        kind,
        subject: subject.to_owned(),
        message: message.map(str::to_owned),
    }
}

#[test]
fn appended_transactions_read_back_in_time_order() {
    let mut builder = builder_with_log();
    let install = transaction(100, TransactionKind::Install, "vim git", Some("dev tools"));
    let uninstall = transaction(2000, TransactionKind::Uninstall, "vim", None);
    let mut store = TransactionStore::write(&mut builder);

    store.append(&uninstall).unwrap();
    store.append(&install).unwrap();
    let tree = builder.commit();

    assert_eq!(TransactionStore::read(&tree).list().unwrap(), vec![install, uninstall]);
}

#[test]
fn a_record_is_named_by_time_and_kind() {
    let mut builder = builder_with_log();

    TransactionStore::write(&mut builder)
        .append(&transaction(42, TransactionKind::Update, "curl", None))
        .unwrap();
    let tree = builder.commit();
    let stat = tree.get_metadata("pkg/upac-tx/42:update").unwrap();

    assert_eq!(
        stat.xattrs.get(OsStr::new("user.upac.tx.subject")).unwrap().as_ref(),
        b"curl"
    );
    assert!(stat.xattrs.get(OsStr::new("user.upac.tx.message")).is_none());
}

#[test]
fn the_same_operation_at_another_time_is_another_record() {
    let mut builder = builder_with_log();
    let mut store = TransactionStore::write(&mut builder);

    store
        .append(&transaction(1, TransactionKind::Install, "vim", None))
        .unwrap();
    store
        .append(&transaction(2, TransactionKind::Install, "vim", None))
        .unwrap();
    let tree = builder.commit();

    assert_eq!(TransactionStore::read(&tree).list().unwrap().len(), 2);
}

#[test]
fn a_tree_without_a_log_has_no_transactions() {
    let tree = TreeBuilder::new().commit();

    assert!(TransactionStore::read(&tree).list().unwrap().is_empty());
}

#[test]
fn an_initialised_log_is_empty() {
    let tree = builder_with_log().commit();

    assert!(TransactionStore::read(&tree).list().unwrap().is_empty());
}

#[test]
fn the_log_needs_the_package_root() {
    let mut builder = TreeBuilder::new();

    assert!(TransactionStore::write(&mut builder).init().is_err());
}

#[test]
fn appending_needs_the_log_directory() {
    let mut builder = TreeBuilder::new();

    assert!(
        TransactionStore::write(&mut builder)
            .append(&transaction(1, TransactionKind::Install, "vim", None))
            .is_err()
    );
}
