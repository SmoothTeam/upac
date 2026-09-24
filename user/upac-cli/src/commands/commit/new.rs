// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::Args as ClapArgs;

use upac_types::request::mutated::CommitRequest;

use crate::types::{CommandContext, call, request_base};

#[derive(ClapArgs)]
pub struct Args {
    pub message: String,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let symbols = ctx.lib.require_write()?;

    let request = CommitRequest {
        base: request_base!(),
        tmp_path: &ctx.tmp_path,
        subject: &args.message,
        message: None,
    };

    call!(symbols.commit, request)
}
