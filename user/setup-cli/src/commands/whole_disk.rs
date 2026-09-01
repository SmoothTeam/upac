// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, bail};

use clap::Args as ClapArgs;

use i18n_embed_fl::fl;

use upac_abi::hook::CancelToken;

use upac_setup::data::SetupWholeDiskData;
use upac_setup::layout::btrfs::{NODE_SIZE, SECTOR_SIZE};

use upac_types::PartitionSpec;

use crate::errors::LocalizedSetupError;
use crate::locale::LOADER;
use crate::progress::{ProgressState, on_progress};
use crate::types::{FsKind, parse_extra_partition, parse_size_mib};

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long)]
    pub device: Option<String>,
    #[arg(long = "esp-size", value_parser = parse_size_mib)]
    pub esp_size_mib: Option<u64>,
    #[arg(long, value_enum)]
    pub deploy_fs: Option<FsKind>,
    #[arg(long = "deploy-size", value_parser = parse_size_mib)]
    pub deploy_size_mib: Option<u64>,
    #[arg(long = "extra-partition", value_parser = parse_extra_partition)]
    pub extra_partitions: Vec<PartitionSpec>,
    #[arg(long)]
    pub force_wipe: bool,
    #[arg(long, default_value_t = NODE_SIZE)]
    pub node_size: u32,
    #[arg(long, default_value_t = SECTOR_SIZE)]
    pub sector_size: u32,

    #[arg(long)]
    pub mount_point: Option<String>,
    #[arg(long)]
    pub source: Option<String>,
    #[arg(long)]
    pub meta_filename: Option<String>,
    #[arg(long)]
    pub empty_config: bool,
    #[arg(long)]
    pub pinned: bool,
    #[arg(long)]
    pub boot_plugin: Option<String>,
}

pub fn run(args: Args, cancel_token: &CancelToken) -> Result<()> {
    let Some(device) = args.device.as_deref() else {
        bail!(fl!(LOADER, "err-missing-device"));
    };
    let Some(esp_size_mib) = args.esp_size_mib else {
        bail!(fl!(LOADER, "err-missing-esp-size"));
    };
    let Some(deploy_fs) = args.deploy_fs else {
        bail!(fl!(LOADER, "err-missing-deploy-fs"));
    };
    let Some(deploy_size_mib) = args.deploy_size_mib else {
        bail!(fl!(LOADER, "err-missing-deploy-size"));
    };
    let Some(source) = args.source.as_deref() else {
        bail!(fl!(LOADER, "err-missing-source"));
    };

    let mut progress = ProgressState::new();

    let data = SetupWholeDiskData {
        device_path: device,
        esp_size_mib,
        deploy_fs: deploy_fs.into(),
        deploy_size_mib,
        extra_partitions: args.extra_partitions,
        force_wipe: args.force_wipe,

        node_size: args.node_size,
        sector_size: args.sector_size,

        mount_point: args.mount_point.as_deref(),
        source,
        meta_filename: args.meta_filename.as_deref(),
        empty_config: args.empty_config,
        pinned: args.pinned,
        boot_plugin: args.boot_plugin.as_deref(),

        hook_message: Some(on_progress),
        hook_message_context: progress.ctx_ptr(),

        cancel_token,
    };

    let result = data.run();
    progress.finish();

    result.map_err(LocalizedSetupError)?;

    Ok(())
}
