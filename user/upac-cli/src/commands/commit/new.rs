// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use std::ffi::CString;
use std::mem::size_of;

use upac_abi::request::CCommitRequest;

use crate::types::CommandContext;
use crate::types::abi::{empty_slice, invoke, request_base, slice_from_cstr};

#[derive(clap::Args)]
pub struct Args {
    pub message: String,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let symbols = ctx.lib.require_write()?;
    let subject = CString::new(args.message)?;

    let request = CCommitRequest {
        struct_size: size_of::<CCommitRequest>(),
        base: request_base(),
        tmp_path: slice_from_cstr(&ctx.tmp_path),
        subject: slice_from_cstr(&subject),
        message: empty_slice(),
    };

    invoke(|error| unsafe { (symbols.commit)(request, error) })
}
