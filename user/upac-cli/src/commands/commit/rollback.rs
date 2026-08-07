// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use std::ffi::CString;
use std::ptr::null_mut;

use crate::cancel_token_ptr;
use crate::ffi::request::CMutatedRequest;
use crate::types::CommandContext;
use crate::types::errors::LibError;

#[derive(clap::Args)]
pub struct Args {
    pub commit: String,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let commit_hash = CString::new(args.commit)?;

    let request = CMutatedRequest::for_rollback(
        &commit_hash,
        &ctx.config.paths.repo_path,
        &ctx.config.paths.root_path,
        &ctx.config.ostree.branch,
        None,
        null_mut(),
        cancel_token_ptr(),
    );

    let return_code = unsafe { (ctx.lib.commit.rollback)(request) };
    LibError::check(return_code)?;

    Ok(())
}
