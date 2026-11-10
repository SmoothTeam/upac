// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::{ArgGroup, Args as ClapArgs};

use i18n_embed_fl::fl;

use upac_types::error::ErrorDomain;
use upac_types::request::bootstrap::SetupBootstrapImportRequest;
use upac_types::response::bootstrap::SetupBootstrapImportResponse;

use crate::libcore::Lib;
use crate::locale::LOADER;
use crate::types::output::Output;
use crate::types::progress::ProgressState;
use crate::types::{query, request_base};

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
    pub source: String,
    #[arg(long)]
    pub empty_config: bool,
}

pub fn run(args: Args, lib: &Lib, output: &Output) -> Result<()> {
    lib.require_root()?;

    let response = {
        let mut progress = ProgressState::new(ErrorDomain::BootstrapImport);

        query!(
            lib.bootstrap_import,
            SetupBootstrapImportRequest {
                base: request_base!(progress),
                disk: args.disk.as_deref(),
                deploy_device: args.deploy_device.as_deref(),
                mount_point: args.mount_point.as_deref(),
                tmp_path: args.tmp_dir.as_deref(),
                source: &args.source,
                empty_config: args.empty_config,
            } => SetupBootstrapImportResponse
        )?
    };

    output.print(
        &[("prefix", &response.prefix_digest), ("config", &response.config_digest)],
        || {
            format!(
                "{}\n{}",
                fl!(
                    LOADER,
                    "bootstrap-prefix-digest",
                    digest = response.prefix_digest.as_str()
                ),
                fl!(
                    LOADER,
                    "bootstrap-config-digest",
                    digest = response.config_digest.as_str()
                )
            )
        },
    );

    Ok(())
}
