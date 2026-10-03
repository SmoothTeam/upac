// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::read;

use tempfile::TempDir;

use upac_types::transaction::{Transaction, TransactionKind};

use upac_composefs::Digest;

use super::super::error::PrefixMetaError;
use super::super::layout::deployment::PREFIX_META_FILENAME;
use super::PrefixDeploy;
use super::config::ConfigDeploy;

fn digest(seed: u8) -> Digest {
    Digest::from_hex(&format!("{seed:02x}").repeat(32)).unwrap()
}

fn transaction() -> Transaction {
    Transaction::new(None, TransactionKind::Install, "install foo".to_owned(), None)
}

fn prefix() -> PrefixDeploy {
    PrefixDeploy::new(
        digest(1),
        transaction(),
        ConfigDeploy::new(digest(11), "install".to_owned(), None),
    )
}

#[test]
fn writing_and_reading_back_keeps_the_mutable_state() {
    let scratch = TempDir::new().unwrap();

    let mut written = prefix();
    written.add_config(ConfigDeploy::new(digest(12), "commit".to_owned(), None));
    written.switch_config(0).unwrap();
    written.set_pinned(true);
    written.write(scratch.path()).unwrap();

    let read_back = PrefixDeploy::read(digest(1), written.transaction().clone(), scratch.path()).unwrap();

    assert_eq!(read_back, written);
}

#[test]
fn the_meta_file_holds_only_the_mutable_state() {
    let scratch = TempDir::new().unwrap();
    prefix().write(scratch.path()).unwrap();

    let content: serde_json::Value =
        serde_json::from_slice(&read(scratch.path().join(PREFIX_META_FILENAME)).unwrap()).unwrap();
    let mut keys: Vec<&str> = content.as_object().unwrap().keys().map(String::as_str).collect();
    keys.sort();

    assert_eq!(keys, vec!["configs", "current_config", "pinned"]);
}

#[test]
fn reading_a_prefix_without_a_meta_file_fails_with_not_found() {
    let scratch = TempDir::new().unwrap();

    let result = PrefixDeploy::read(digest(1), transaction(), scratch.path());

    assert_eq!(result.err(), Some(PrefixMetaError::NotFound));
}
