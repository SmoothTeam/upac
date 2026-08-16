// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use upac_abi::request::CGcRequest;

use crate::types::CommandContext;
use crate::types::abi::{invoke, request_base};

#[derive(clap::Args)]
pub struct Args {}

pub fn run(_args: Args, ctx: CommandContext) -> Result<()> {
    let symbols = ctx.lib.require_write()?;

    let request = CGcRequest::new(request_base());

    invoke(|error| unsafe { (symbols.gc)(request, error) })
}
