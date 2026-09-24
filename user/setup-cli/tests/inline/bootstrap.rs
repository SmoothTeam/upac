// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use upac_abi::FsKind as FsKindAbi;
use upac_abi::InitramfsGenerator;

use crate::libcore::Lib;
use crate::locale;
use crate::types::{BootPlugin, FsKind, InitramfsGeneratorClapArg};

use super::{Args, run};

fn valid_args() -> Args {
    Args {
        esp_device: Some("/dev/sda1".to_owned()),
        deploy_device: Some("/dev/sda2".to_owned()),
        deploy_fs: FsKind(FsKindAbi::Btrfs),
        mount_point: None,
        source: Some("/mnt/source".to_owned()),
        empty_config: false,
        pinned: false,
        boot_plugin: BootPlugin::SystemdBoot,
        initramfs_generator: InitramfsGeneratorClapArg(InitramfsGenerator::Dracut),
    }
}

fn run_bailing(args: Args) -> String {
    locale::init_for_test();
    let lib = Lib::load().unwrap();

    run(args, &lib).unwrap_err().to_string()
}

#[test]
fn missing_esp_device_bails_before_touching_the_disk() {
    let message = run_bailing(Args {
        esp_device: None,
        ..valid_args()
    });

    assert_eq!(message, "Missing required argument: --esp-device");
}

#[test]
fn missing_deploy_device_bails_before_touching_the_disk() {
    let message = run_bailing(Args {
        deploy_device: None,
        ..valid_args()
    });

    assert_eq!(message, "Missing required argument: --deploy-device");
}

#[test]
fn missing_source_bails_before_touching_the_disk() {
    let message = run_bailing(Args {
        source: None,
        ..valid_args()
    });

    assert_eq!(message, "Missing required argument: --source");
}
