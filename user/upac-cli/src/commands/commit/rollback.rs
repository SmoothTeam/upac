// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use std::ffi::CString;
use std::mem::size_of;

use upac_abi::request::CRollbackRequest;

use crate::types::CommandContext;
use crate::types::abi::{invoke, request_base, slice_from_cstr};

#[derive(clap::Args)]
pub struct Args {
    pub commit: String,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let symbols = ctx.lib.require_write()?;
    let config_digest = CString::new(args.commit)?;

    let request = CRollbackRequest {
        struct_size: size_of::<CRollbackRequest>(),
        base: request_base(),
        tmp_path: slice_from_cstr(&ctx.tmp_path),
        config_digest: slice_from_cstr(&config_digest),
    };

    invoke(|error| unsafe { (symbols.rollback)(request, error) })
}
