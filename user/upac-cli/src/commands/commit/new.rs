// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ptr::null_mut;

use anyhow::Result;

use clap::Args as ClapArgs;

use upac_types::request::RequestBase;
use upac_types::request::mutated::CommitRequest;

use crate::cancel_token_ptr;
use crate::types::CommandContext;
use crate::types::abi::invoke;

#[derive(ClapArgs)]
pub struct Args {
    pub message: String,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let symbols = ctx.lib.require_write()?;

    let request = CommitRequest {
        base: RequestBase {
            on_hook: None,
            hook_ctx: null_mut(),
            cancel_token: cancel_token_ptr(),
        },
        tmp_path: &ctx.tmp_path.to_string_lossy(),
        subject: &args.message,
        message: None,
    }
    .into();

    let result = invoke(|error| unsafe { (symbols.commit)(request, error) });
    unsafe { request.free() }

    result
}
