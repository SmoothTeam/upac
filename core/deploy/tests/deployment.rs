// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::transaction::{Transaction, TransactionKind};

use upac_composefs::Digest;

use upac_deploy::deployment::config::ConfigDeploy;
use upac_deploy::deployment::{Deployment, PrefixDeploy};
use upac_deploy::error::PrefixDeployError;

fn digest(seed: u8) -> Digest {
    Digest::from_hex(&format!("{seed:02x}").repeat(32)).unwrap()
}

fn config(seed: u8) -> ConfigDeploy {
    ConfigDeploy::new(digest(seed), "commit".to_owned(), None)
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
    PrefixDeploy::new(digest(1), transaction(), config(11))
}

#[test]
fn a_new_prefix_starts_with_its_first_config_as_current_and_unpinned() {
    let prefix = prefix();

    assert_eq!(prefix.configs().len(), 1);
    assert_eq!(prefix.current_config().map(Deployment::digest), Some(&digest(11)));
    assert!(!prefix.pinned());
}

#[test]
fn prefix_metadata_comes_from_its_transaction() {
    let prefix = prefix();

    assert_eq!(prefix.digest(), &digest(1));
    assert_eq!(prefix.subject(), "install foo");
    assert_eq!(prefix.message(), Some("because"));
    assert_eq!(prefix.timestamp(), prefix.transaction().timestamp);
}

#[test]
fn adding_a_config_makes_it_current() {
    let mut prefix = prefix();
    prefix.add_config(config(12));

    assert_eq!(prefix.configs().len(), 2);
    assert_eq!(prefix.current_config().map(Deployment::digest), Some(&digest(12)));
}

#[test]
fn switching_to_an_existing_config_makes_it_current() {
    let mut prefix = prefix();
    prefix.add_config(config(12));

    assert_eq!(prefix.switch_config(0), Ok(()));
    assert_eq!(prefix.current_config().map(Deployment::digest), Some(&digest(11)));
}

#[test]
fn switching_to_a_missing_config_fails_and_keeps_the_current_one() {
    let mut prefix = prefix();

    assert_eq!(prefix.switch_config(5), Err(PrefixDeployError::ConfigNotFound(5)));
    assert_eq!(prefix.current_config().map(Deployment::digest), Some(&digest(11)));
}

#[test]
fn referenced_trees_are_the_prefix_and_all_of_its_configs() {
    let mut prefix = prefix();
    prefix.add_config(config(12));

    assert_eq!(prefix.referenced_trees(), vec![&digest(1), &digest(11), &digest(12)]);
}
