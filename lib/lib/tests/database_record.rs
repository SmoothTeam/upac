// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use tempfile::{Builder, TempDir};
use upac::database::error::DeployRecordError;
use upac::database::record::{ConfigHistoryEntry, DeployRecord};

fn scratch_dir(name: &str) -> TempDir {
    Builder::new().prefix(name).tempdir().unwrap()
}

fn sample_record() -> DeployRecord {
    DeployRecord {
        prefix_digest: "usr-digest-abc123".to_owned(),
        subject: "install firefox".to_owned(),
        message: Some("long-form commit message".to_owned()),
        seq: 7,
        timestamp: 1_754_000_000,
        config_history: vec![
            ConfigHistoryEntry {
                config_digest: "etc-digest-1".to_owned(),
                subject: "first etc".to_owned(),
                message: None,
            },
            ConfigHistoryEntry {
                config_digest: "etc-digest-2".to_owned(),
                subject: "second etc".to_owned(),
                message: Some("with a message".to_owned()),
            },
        ],
        working_config: "etc-digest-2".to_owned(),
        pinned: false,
    }
}

#[test]
fn deploy_record_json_round_trip_preserves_value() {
    let record = sample_record();

    let value = record.to_json();
    let decoded = DeployRecord::from_json(&value).unwrap();

    assert_eq!(record, decoded);
}

#[test]
fn deploy_record_disk_round_trip_preserves_value() {
    let dir = scratch_dir("disk-round-trip");
    let record = sample_record();

    record.write(dir.path()).unwrap();
    let decoded = DeployRecord::read(dir.path()).unwrap();

    assert_eq!(record, decoded);
}

#[test]
fn deploy_record_read_fails_when_file_missing() {
    let dir = scratch_dir("missing-file");

    assert!(matches!(
        DeployRecord::read(dir.path()),
        Err(DeployRecordError::NotFound)
    ));
}

#[test]
fn deploy_record_from_json_fails_on_non_object() {
    let value = serde_json::Value::String("not an object".to_owned());

    assert!(matches!(
        DeployRecord::from_json(&value),
        Err(DeployRecordError::InvalidField)
    ));
}

#[test]
fn deploy_record_from_json_fails_on_missing_field() {
    let mut object = sample_record().to_json();
    object.as_object_mut().unwrap().remove("prefix_digest");

    assert!(matches!(
        DeployRecord::from_json(&object),
        Err(DeployRecordError::InvalidField)
    ));
}

#[test]
fn deploy_record_from_json_defaults_pinned_to_false_when_absent() {
    let mut object = sample_record().to_json();
    object.as_object_mut().unwrap().remove("pinned");

    let decoded = DeployRecord::from_json(&object).unwrap();

    assert!(!decoded.pinned);
}

#[test]
fn deploy_record_write_overwrites_previous_value() {
    let dir = scratch_dir("overwrite");

    sample_record().write(dir.path()).unwrap();

    let mut second = sample_record();
    second.seq = 42;
    second.write(dir.path()).unwrap();

    let decoded = DeployRecord::read(dir.path()).unwrap();
    assert_eq!(decoded.seq, 42);
}
