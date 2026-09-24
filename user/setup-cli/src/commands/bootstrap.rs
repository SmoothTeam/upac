// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, bail};

use clap::{Args as ClapArgs, ValueEnum};

use i18n_embed_fl::fl;

use upac_abi::FsKind as FsKindAbi;
use upac_abi::InitramfsGenerator;
use upac_abi::error::ErrorDomain;

use upac_types::request::bootstrap::SetupBootstrapRequest;

use crate::layout::disk_defaults::DEPLOY_FS;
use crate::layout::initramfs::GENERATOR;
use crate::libcore::Lib;
use crate::locale::LOADER;
use crate::types::progress::ProgressState;
use crate::types::{BootPlugin, FsKind, InitramfsGeneratorClapArg, call, request_base};

#[cfg(test)]
#[path = "../../tests/inline/bootstrap.rs"]
mod tests;

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long)]
    pub esp_device: Option<String>,
    #[arg(long)]
    pub deploy_device: Option<String>,
    #[arg(long, value_enum, default_value_t = FsKind::from_str(DEPLOY_FS, false).unwrap_or(FsKind(FsKindAbi::Btrfs)))]
    pub deploy_fs: FsKind,

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
    #[arg(long, value_enum, default_value_t = InitramfsGeneratorClapArg::from_str(GENERATOR, false).unwrap_or(InitramfsGeneratorClapArg(InitramfsGenerator::Dracut)))]
    pub initramfs_generator: InitramfsGeneratorClapArg,
}

pub fn run(args: Args, lib: &Lib) -> Result<()> {
    let Some(esp_device) = args.esp_device else {
        bail!(fl!(LOADER, "err-missing-esp-device"));
    };
    let Some(deploy_device) = args.deploy_device else {
        bail!(fl!(LOADER, "err-missing-deploy-device"));
    };
    let Some(source) = args.source else {
        bail!(fl!(LOADER, "err-missing-source"));
    };

    lib.require_root()?;

    let mut progress = ProgressState::new(ErrorDomain::Bootstrap);

    call!(
        lib.bootstrap_system,
        SetupBootstrapRequest {
            base: request_base!(progress),
            esp_device: &esp_device,
            deploy_device: &deploy_device,
            deploy_fs: args.deploy_fs.into(),
            mount_point: args.mount_point.as_deref(),
            source: &source,
            empty_config: args.empty_config,
            pinned: args.pinned,
            boot_plugin: args.boot_plugin.as_str(),
            initramfs_generator: args.initramfs_generator.into(),
        }
    )
}
