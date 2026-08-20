// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::CString;

use anyhow::Result;

use clap::Args as ClapArgs;

use upac_abi::request::CCommitRequest;

use crate::types::CommandContext;
use crate::types::abi::{empty_slice, invoke, request_base, slice_from_cstr};

#[derive(ClapArgs)]
pub struct Args {
    pub message: String,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let symbols = ctx.lib.require_write()?;
    let subject = CString::new(args.message)?;

    let request = CCommitRequest::new(
        request_base(),
        slice_from_cstr(&ctx.tmp_path),
        slice_from_cstr(&subject),
        empty_slice(),
    );

    invoke(|error| unsafe { (symbols.commit)(request, error) })
}
