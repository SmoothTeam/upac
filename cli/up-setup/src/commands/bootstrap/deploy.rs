// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::{ArgGroup, Args as ClapArgs};

use upac_types::error::ErrorDomain;
use upac_types::request::bootstrap::SetupBootstrapDeployRequest;

use crate::libcore::Lib;
use crate::types::progress::ProgressState;
use crate::types::{BootPlugin, call, request_base};

#[derive(ClapArgs)]
#[command(group(ArgGroup::new("target").required(true).args(["disk", "esp_device"])))]
pub struct Args {
    #[arg(long, help_heading = "target", conflicts_with_all = ["esp_device", "deploy_device"])]
    pub disk: Option<String>,
    #[arg(long, help_heading = "target", requires = "deploy_device")]
    pub esp_device: Option<String>,
    #[arg(long, help_heading = "target", requires = "esp_device")]
    pub deploy_device: Option<String>,

    #[arg(long)]
    pub mount_point: Option<String>,
    #[arg(long)]
    pub prefix: String,
    #[arg(long)]
    pub config: String,
    #[arg(long, value_enum, default_value_t = BootPlugin::SystemdBoot)]
    pub boot_plugin: BootPlugin,
    #[arg(long)]
    pub pinned: bool,
}

pub fn run(args: Args, lib: &Lib) -> Result<()> {
    lib.require_root()?;

    let mut progress = ProgressState::new(ErrorDomain::BootstrapDeploy);

    call!(
        lib.bootstrap_deploy,
        SetupBootstrapDeployRequest {
            base: request_base!(progress),
            disk: args.disk.as_deref(),
            esp_device: args.esp_device.as_deref(),
            deploy_device: args.deploy_device.as_deref(),
            mount_point: args.mount_point.as_deref(),
            prefix_digest: &args.prefix,
            config_digest: &args.config,
            boot_plugin: args.boot_plugin.as_str(),
            pinned: args.pinned,
        }
    )
}
