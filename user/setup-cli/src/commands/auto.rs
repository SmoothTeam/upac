// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, bail};

use clap::{Args as ClapArgs, ValueEnum};

use i18n_embed_fl::fl;

use upac_abi::FsKind as FsKindAbi;
use upac_abi::InitramfsGenerator;

use upac_types::request::{BtrfsOptions, GptLayout, PartitionSpec, RequestBase, SetupWholeDiskRequest};

use crate::cancel_token_ptr;
use crate::layout::{disk_defaults, initramfs};
use crate::libcore::{Lib, invoke};
use crate::locale::LOADER;
use crate::types::progress::{ProgressState, on_progress};
use crate::types::{BootPlugin, FsKind, InitramfsGeneratorClapArg, parse_extra_partition, parse_size_mib};

#[cfg(test)]
#[path = "../../tests/inline/auto.rs"]
mod tests;

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long)]
    pub device: Option<String>,
    #[arg(long = "esp-size", value_parser = parse_size_mib, default_value_t = u64::from(disk_defaults::ESP_SIZE_MIB))]
    pub esp_size_mib: u64,
    #[arg(long, value_enum, default_value_t = FsKind::from_str(disk_defaults::DEPLOY_FS, false).unwrap_or(FsKind(FsKindAbi::Btrfs)))]
    pub deploy_fs: FsKind,
    #[arg(long = "deploy-size", value_parser = parse_size_mib)]
    pub deploy_size_mib: Option<u64>,
    #[arg(long = "extra-partition", value_parser = parse_extra_partition)]
    pub extra_partitions: Vec<PartitionSpec>,
    #[arg(long)]
    pub force_wipe: bool,
    #[arg(long, default_value_t = disk_defaults::NODE_SIZE)]
    pub node_size: u32,
    #[arg(long, default_value_t = disk_defaults::SECTOR_SIZE)]
    pub sector_size: u32,

    #[arg(long)]
    pub mount_point: Option<String>,
    #[arg(long)]
    pub source: Option<String>,
    #[arg(long)]
    pub empty_config: bool,
    #[arg(long)]
    pub pinned: bool,
    #[arg(long, value_enum, default_value_t = BootPlugin::SystemdBoot)]
    pub boot_plugin: BootPlugin,
    #[arg(long, value_enum, default_value_t = InitramfsGeneratorClapArg::from_str(initramfs::GENERATOR, false).unwrap_or(InitramfsGeneratorClapArg(InitramfsGenerator::Dracut)))]
    pub initramfs_generator: InitramfsGeneratorClapArg,
}

pub fn run(args: Args, lib: &Lib) -> Result<()> {
    let Some(device) = args.device else {
        bail!(fl!(LOADER, "err-missing-device"));
    };
    let Some(deploy_size_mib) = args.deploy_size_mib else {
        bail!(fl!(LOADER, "err-missing-deploy-size"));
    };
    let Some(source) = args.source else {
        bail!(fl!(LOADER, "err-missing-source"));
    };

    lib.require_root()?;

    let mut progress = ProgressState::new();

    let request = SetupWholeDiskRequest {
        base: RequestBase {
            on_hook: Some(on_progress),
            hook_ctx: progress.ctx_ptr(),
            cancel_token: cancel_token_ptr(),
        },

        device_path: device,
        gpt: GptLayout {
            esp_size_mib: args.esp_size_mib,
            deploy_fs: args.deploy_fs.into(),
            deploy_size_mib,
            extra_partitions: args.extra_partitions,
            force_wipe: args.force_wipe,
        },
        btrfs: BtrfsOptions {
            node_size: args.node_size,
            sector_size: args.sector_size,
        },

        mount_point: args.mount_point,
        source,
        empty_config: args.empty_config,
        pinned: args.pinned,
        boot_plugin: args.boot_plugin.as_str().to_owned(),
        initramfs_generator: args.initramfs_generator.into(),
    }
    .into();

    let result = invoke(|error| unsafe { (lib.setup_whole_disk)(request, error) });
    progress.finish();

    result
}
