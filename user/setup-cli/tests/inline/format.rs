// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use upac_abi::FsKind as FsKindAbi;

use crate::libcore::Lib;
use crate::locale;
use crate::types::FsKind;

use super::{Args, CreateArgs, EspArgs, FormatCommand, run};

fn run_bailing(command: FormatCommand) -> String {
    locale::init_for_test();
    let lib = Lib::load().unwrap();

    run(Args { command }, &lib).unwrap_err().to_string()
}

#[test]
fn esp_without_device_bails_before_touching_the_disk() {
    let message = run_bailing(FormatCommand::Esp(EspArgs {
        device: None,
        label: "ESP".to_owned(),
        force_wipe: false,
    }));

    assert_eq!(message, "Missing required argument: --device");
}

#[test]
fn create_without_device_bails_before_touching_the_disk() {
    let message = run_bailing(FormatCommand::Create(CreateArgs {
        device: None,
        fs: FsKind(FsKindAbi::Btrfs),
        label: None,
        force_wipe: false,
        btrfs_node_size: 16384,
        btrfs_sector_size: 4096,
    }));

    assert_eq!(message, "Missing required argument: --device");
}
