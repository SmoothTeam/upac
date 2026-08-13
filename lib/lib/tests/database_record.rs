// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs::{create_dir_all, remove_dir_all};
use std::path::PathBuf;

use upac::database::error::DeployRecordError;
use upac::database::record::{DeployRecord, EtcHistoryEntry};

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("upac-test-database-record-{}-{name}", std::process::id()));
    create_dir_all(&dir).unwrap();

    dir
}

fn sample_record() -> DeployRecord {
    DeployRecord {
        prefix_digest: "usr-digest-abc123".to_string(),
        subject: "install firefox".to_string(),
        message: Some("long-form commit message".to_string()),
        seq: 7,
        timestamp: 1_754_000_000,
        config_history: vec![
            EtcHistoryEntry {
                etc_digest: "etc-digest-1".to_string(),
                subject: "first etc".to_string(),
                message: None,
            },
            EtcHistoryEntry {
                etc_digest: "etc-digest-2".to_string(),
                subject: "second etc".to_string(),
                message: Some("with a message".to_string()),
            },
        ],
        working_etc: "etc-digest-2".to_string(),
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

    record.write(&dir).unwrap();
    let decoded = DeployRecord::read(&dir).unwrap();

    assert_eq!(record, decoded);

    let _ = remove_dir_all(&dir);
}

#[test]
fn deploy_record_read_fails_when_file_missing() {
    let dir = scratch_dir("missing-file");

    assert!(matches!(DeployRecord::read(&dir), Err(DeployRecordError::NotFound)));

    let _ = remove_dir_all(&dir);
}

#[test]
fn deploy_record_from_json_fails_on_non_object() {
    let value = serde_json::Value::String("not an object".to_string());

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
fn deploy_record_write_overwrites_previous_value() {
    let dir = scratch_dir("overwrite");

    sample_record().write(&dir).unwrap();

    let mut second = sample_record();
    second.seq = 42;
    second.write(&dir).unwrap();

    let decoded = DeployRecord::read(&dir).unwrap();
    assert_eq!(decoded.seq, 42);

    let _ = remove_dir_all(&dir);
}
