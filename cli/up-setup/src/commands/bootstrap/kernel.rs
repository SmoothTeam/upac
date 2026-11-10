// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::{ArgGroup, Args as ClapArgs, ValueEnum};

use i18n_embed_fl::fl;

use upac_types::error::ErrorDomain;
use upac_types::request::bootstrap::{InitramfsGenerator, SetupBootstrapKernelRequest};
use upac_types::response::bootstrap::SetupBootstrapKernelResponse;

use crate::layout::initramfs::GENERATOR;
use crate::libcore::Lib;
use crate::locale::LOADER;
use crate::types::output::Output;
use crate::types::progress::ProgressState;
use crate::types::{InitramfsGeneratorClapArg, query, request_base};

#[derive(ClapArgs)]
#[command(group(ArgGroup::new("target").required(true).args(["disk", "deploy_device"])))]
pub struct Args {
    #[arg(long, help_heading = "root-target")]
    pub disk: Option<String>,
    #[arg(long, help_heading = "root-target")]
    pub deploy_device: Option<String>,

    #[arg(long)]
    pub mount_point: Option<String>,
    #[arg(long)]
    pub tmp_dir: Option<String>,
    #[arg(long)]
    pub prefix: String,
    #[arg(long, value_enum, default_value_t = InitramfsGeneratorClapArg::from_str(GENERATOR, false).unwrap_or(InitramfsGeneratorClapArg(InitramfsGenerator::Dracut)))]
    pub initramfs_generator: InitramfsGeneratorClapArg,
}

pub fn run(args: Args, lib: &Lib, output: &Output) -> Result<()> {
    lib.require_root()?;

    let response = {
        let mut progress = ProgressState::new(ErrorDomain::BootstrapKernel);

        query!(
            lib.bootstrap_kernel,
            SetupBootstrapKernelRequest {
                base: request_base!(progress),
                disk: args.disk.as_deref(),
                deploy_device: args.deploy_device.as_deref(),
                mount_point: args.mount_point.as_deref(),
                tmp_path: args.tmp_dir.as_deref(),
                prefix_digest: &args.prefix,
                initramfs_generator: args.initramfs_generator.into(),
            } => SetupBootstrapKernelResponse
        )?
    };

    output.print(&[("prefix", &response.prefix_digest)], || {
        fl!(
            LOADER,
            "bootstrap-prefix-digest",
            digest = response.prefix_digest.as_str()
        )
    });

    Ok(())
}
