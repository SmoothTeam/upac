// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use upac_abi::PartitionKind;

use crate::libcore::Lib;
use crate::locale;
use crate::types::PartitionKindClapArg;

use super::{Args, CreateArgs, EspArgs, InitArgs, PartitionCommand, run};

fn run_bailing(command: PartitionCommand) -> String {
    locale::init_for_test();
    let lib = Lib::load().unwrap();

    run(Args { command }, &lib).unwrap_err().to_string()
}

fn valid_create_args() -> CreateArgs {
    CreateArgs {
        device: Some("/dev/sda".to_owned()),
        size_mib: Some(8192),
        label: Some("root".to_owned()),
        kind: PartitionKindClapArg(PartitionKind::Root),
    }
}

#[test]
fn init_without_device_bails_before_touching_the_disk() {
    let message = run_bailing(PartitionCommand::Init(InitArgs {
        device: None,
        force_wipe: false,
    }));

    assert_eq!(message, "Missing required argument: --device");
}

#[test]
fn esp_without_device_bails_before_touching_the_disk() {
    let message = run_bailing(PartitionCommand::Esp(EspArgs {
        device: None,
        size_mib: 256,
        label: "ESP".to_owned(),
    }));

    assert_eq!(message, "Missing required argument: --device");
}

#[test]
fn create_without_device_bails_before_touching_the_disk() {
    let message = run_bailing(PartitionCommand::Create(CreateArgs {
        device: None,
        ..valid_create_args()
    }));

    assert_eq!(message, "Missing required argument: --device");
}

#[test]
fn create_without_size_bails_before_touching_the_disk() {
    let message = run_bailing(PartitionCommand::Create(CreateArgs {
        size_mib: None,
        ..valid_create_args()
    }));

    assert_eq!(message, "Missing required argument: --size");
}

#[test]
fn create_without_label_bails_before_touching_the_disk() {
    let message = run_bailing(PartitionCommand::Create(CreateArgs {
        label: None,
        ..valid_create_args()
    }));

    assert_eq!(message, "Missing required argument: --label");
}
