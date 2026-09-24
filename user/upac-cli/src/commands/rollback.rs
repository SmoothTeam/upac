// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::Args as ClapArgs;

use upac_types::request::mutated::RollbackRequest;

use crate::types::{CommandContext, boot_plugin, call, request_base};

#[derive(ClapArgs)]
pub struct Args {
    pub commit: String,
    #[arg(long)]
    pub boot: Option<String>,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let symbols = ctx.lib.require_write()?;

    let boot_plugin = boot_plugin!(args.boot)?;

    let request = RollbackRequest {
        base: request_base!(),
        tmp_path: &ctx.tmp_path,
        config_digest: &args.commit,
        boot_plugin: &boot_plugin,
    };

    call!(symbols.rollback, request)
}
