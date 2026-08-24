// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::CString;

use anyhow::Result;

use clap::Args as ClapArgs;

use upac_abi::request::CPinRequest;

use crate::types::CommandContext;
use crate::types::abi::{invoke, request_base, slice_from_cstr};

#[derive(ClapArgs)]
pub struct Args {
    pub digest: String,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let symbols = ctx.lib.require_write()?;
    let prefix_digest = CString::new(args.digest)?;

    let request = CPinRequest::new(request_base(), slice_from_cstr(&prefix_digest), false);

    invoke(|error| unsafe { (symbols.pin_deploy)(request, error) })
}
