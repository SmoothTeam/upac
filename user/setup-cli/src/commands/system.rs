// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::{Args as ClapArgs, ValueEnum};

use upac_abi::InitramfsGenerator;

use upac_types::request::{PartitionMount, RequestBase, SetupExistingRequest};

use crate::cancel_token_ptr;
use crate::layout::initramfs::GENERATOR;
use crate::libcore::{Lib, invoke};
use crate::types::progress::{ProgressState, on_progress};
use crate::types::{BootPlugin, FsKind, InitramfsGeneratorClapArg, parse_extra_mount};

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long)]
    pub esp_device: String,
    #[arg(long)]
    pub deploy_device: String,
    #[arg(long, value_enum)]
    pub deploy_fs: FsKind,
    #[arg(long = "extra-mount", value_parser = parse_extra_mount)]
    pub extra_mounts: Vec<PartitionMount>,

    #[arg(long)]
    pub mount_point: Option<String>,
    #[arg(long)]
    pub source: String,
    #[arg(long)]
    pub empty_config: bool,
    #[arg(long)]
    pub pinned: bool,
    #[arg(long, value_enum, default_value_t = BootPlugin::SystemdBoot)]
    pub boot_plugin: BootPlugin,
    #[arg(long, value_enum, default_value_t = InitramfsGeneratorClapArg::from_str(GENERATOR, false).unwrap_or(InitramfsGeneratorClapArg(InitramfsGenerator::Dracut)))]
    pub initramfs_generator: InitramfsGeneratorClapArg,
}

pub fn run(args: Args, lib: &Lib) -> Result<()> {
    lib.require_root()?;

    let mut progress = ProgressState::new();

    let request = SetupExistingRequest {
        base: RequestBase {
            on_hook: Some(on_progress),
            hook_ctx: progress.ctx_ptr(),
            cancel_token: cancel_token_ptr(),
        },

        esp_device: args.esp_device,
        deploy_device: args.deploy_device,
        deploy_fs: args.deploy_fs.into(),
        extra_mounts: args.extra_mounts,

        mount_point: args.mount_point,
        source: args.source,
        empty_config: args.empty_config,
        pinned: args.pinned,
        boot_plugin: args.boot_plugin.as_str().to_owned(),
        initramfs_generator: args.initramfs_generator.into(),
    }
    .into();

    let result = invoke(|error| unsafe { (lib.setup_existing)(request, error) });
    progress.finish();

    result
}
