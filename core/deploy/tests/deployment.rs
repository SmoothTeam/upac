// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::transaction::{Transaction, TransactionKind};

use upac_deploy::deployment::config::ConfigDeploy;
use upac_deploy::deployment::{Deployment, PrefixDeploy};
use upac_deploy::error::PrefixDeployError;

fn config(digest: &str) -> ConfigDeploy {
    ConfigDeploy::new(digest.to_owned(), "commit".to_owned(), None)
}

fn transaction() -> Transaction {
    Transaction::new(
        Some("parent-digest".to_owned()),
        TransactionKind::Install,
        "install foo".to_owned(),
        Some("because".to_owned()),
    )
}

fn prefix() -> PrefixDeploy {
    PrefixDeploy::new("prefix-digest".to_owned(), transaction(), config("config-1"))
}

#[test]
fn a_new_prefix_starts_with_its_first_config_as_current_and_unpinned() {
    let prefix = prefix();

    assert_eq!(prefix.configs().len(), 1);
    assert_eq!(prefix.current_config().map(Deployment::digest), Some("config-1"));
    assert!(!prefix.pinned());
}

#[test]
fn prefix_metadata_comes_from_its_transaction() {
    let prefix = prefix();

    assert_eq!(prefix.digest(), "prefix-digest");
    assert_eq!(prefix.subject(), "install foo");
    assert_eq!(prefix.message(), Some("because"));
    assert_eq!(prefix.timestamp(), prefix.transaction().timestamp);
}

#[test]
fn adding_a_config_makes_it_current() {
    let mut prefix = prefix();
    prefix.add_config(config("config-2"));

    assert_eq!(prefix.configs().len(), 2);
    assert_eq!(prefix.current_config().map(Deployment::digest), Some("config-2"));
}

#[test]
fn switching_to_an_existing_config_makes_it_current() {
    let mut prefix = prefix();
    prefix.add_config(config("config-2"));

    assert_eq!(prefix.switch_config(0), Ok(()));
    assert_eq!(prefix.current_config().map(Deployment::digest), Some("config-1"));
}

#[test]
fn switching_to_a_missing_config_fails_and_keeps_the_current_one() {
    let mut prefix = prefix();

    assert_eq!(prefix.switch_config(5), Err(PrefixDeployError::ConfigNotFound(5)));
    assert_eq!(prefix.current_config().map(Deployment::digest), Some("config-1"));
}

#[test]
fn referenced_trees_are_the_prefix_and_all_of_its_configs() {
    let mut prefix = prefix();
    prefix.add_config(config("config-2"));

    assert_eq!(prefix.referenced_trees(), vec!["prefix-digest", "config-1", "config-2"]);
}
