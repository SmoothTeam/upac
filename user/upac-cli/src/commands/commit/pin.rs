// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::Args as ClapArgs;

use upac_types::request::mutated::PinRequest;

use crate::types::{CommandContext, call, request_base};

#[derive(ClapArgs)]
pub struct Args {
    pub digest: String,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let symbols = ctx.lib.require_write()?;

    let request = PinRequest {
        base: request_base!(),
        prefix_digest: &args.digest,
        pinned: true,
    };

    call!(symbols.pin_deploy, request)
}
