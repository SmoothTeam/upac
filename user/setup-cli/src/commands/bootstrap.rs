// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::{ArgGroup, Args as ClapArgs, ValueEnum};

use upac_abi::FsKind as FsKindAbi;
use upac_abi::InitramfsGenerator;
use upac_abi::error::ErrorDomain;

use upac_types::request::bootstrap::SetupBootstrapRequest;

use crate::layout::disk_defaults::DEPLOY_FS;
use crate::layout::initramfs::GENERATOR;
use crate::libcore::Lib;
use crate::types::progress::ProgressState;
use crate::types::{BootPlugin, FsKind, InitramfsGeneratorClapArg, call, request_base};

#[derive(ClapArgs)]
#[command(group(ArgGroup::new("target").required(true).args(["disk", "esp_device"])))]
pub struct Args {
    #[arg(long, conflicts_with_all = ["esp_device", "deploy_device"])]
    pub disk: Option<String>,
    #[arg(long, requires = "deploy_device")]
    pub esp_device: Option<String>,
    #[arg(long, requires = "esp_device")]
    pub deploy_device: Option<String>,
    #[arg(long, value_enum, default_value_t = FsKind::from_str(DEPLOY_FS, false).unwrap_or(FsKind(FsKindAbi::Btrfs)))]
    pub deploy_fs: FsKind,

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

    let mut progress = ProgressState::new(ErrorDomain::Bootstrap);

    call!(
        lib.bootstrap_system,
        SetupBootstrapRequest {
            base: request_base!(progress),
            disk: args.disk.as_deref(),
            esp_device: args.esp_device.as_deref(),
            deploy_device: args.deploy_device.as_deref(),
            deploy_fs: args.deploy_fs.into(),
            mount_point: args.mount_point.as_deref(),
            source: &args.source,
            empty_config: args.empty_config,
            pinned: args.pinned,
            boot_plugin: args.boot_plugin.as_str(),
            initramfs_generator: args.initramfs_generator.into(),
        }
    )
}
