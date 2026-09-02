// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use upac_abi::FsKind as FsKindAbi;
use upac_abi::hook::CancelToken;

use crate::locale;
use crate::types::FsKind;

use super::{Args, run};

fn valid_args() -> Args {
    Args {
        device: Some("/dev/sda".to_owned()),
        esp_size_mib: Some(512),
        deploy_fs: Some(FsKind(FsKindAbi::Ext4)),
        deploy_size_mib: Some(8192),
        extra_partitions: Vec::new(),
        force_wipe: false,
        node_size: 0,
        sector_size: 0,

        mount_point: None,
        source: Some("/mnt/source".to_owned()),
        meta_filename: None,
        empty_config: false,
        pinned: false,
        boot_plugin: None,
    }
}

#[test]
fn missing_device_bails_before_touching_the_disk() {
    locale::init_for_test();
    let cancel_token = CancelToken::new();

    let error = run(
        Args {
            device: None,
            ..valid_args()
        },
        &cancel_token,
    )
    .unwrap_err();

    assert_eq!(error.to_string(), "Missing required argument: --device");
}

#[test]
fn missing_esp_size_bails_before_touching_the_disk() {
    locale::init_for_test();
    let cancel_token = CancelToken::new();

    let error = run(
        Args {
            esp_size_mib: None,
            ..valid_args()
        },
        &cancel_token,
    )
    .unwrap_err();

    assert_eq!(error.to_string(), "Missing required argument: --esp-size");
}

#[test]
fn missing_deploy_fs_bails_before_touching_the_disk() {
    locale::init_for_test();
    let cancel_token = CancelToken::new();

    let error = run(
        Args {
            deploy_fs: None,
            ..valid_args()
        },
        &cancel_token,
    )
    .unwrap_err();

    assert_eq!(error.to_string(), "Missing required argument: --deploy-fs");
}

#[test]
fn missing_deploy_size_bails_before_touching_the_disk() {
    locale::init_for_test();
    let cancel_token = CancelToken::new();

    let error = run(
        Args {
            deploy_size_mib: None,
            ..valid_args()
        },
        &cancel_token,
    )
    .unwrap_err();

    assert_eq!(error.to_string(), "Missing required argument: --deploy-size");
}

#[test]
fn missing_source_bails_before_touching_the_disk() {
    locale::init_for_test();
    let cancel_token = CancelToken::new();

    let error = run(
        Args {
            source: None,
            ..valid_args()
        },
        &cancel_token,
    )
    .unwrap_err();

    assert_eq!(error.to_string(), "Missing required argument: --source");
}
