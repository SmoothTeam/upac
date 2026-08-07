// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use std::ffi::CString;

use colored::Colorize;

use crate::cancel_token_ptr;
use crate::ffi::ctypes::CDiffKind;
use crate::ffi::request::{CUnmutatedRequest, CUnmutatedResponse};
use crate::types::CommandContext;
use crate::types::errors::LibError;

#[derive(clap::Args)]
pub struct Args {
    pub from: String,
    pub to: String,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let from_commit_hash = CString::new(args.from)?;
    let to_commit_hash = CString::new(args.to)?;

    let mut response = CUnmutatedResponse::empty();

    let request = CUnmutatedRequest::for_diff(
        &ctx.config.paths.repo_path,
        &ctx.tmp_path,
        &from_commit_hash,
        &to_commit_hash,
        cancel_token_ptr(),
    );

    let return_code = unsafe { (ctx.lib.pkg.diff)(request, &mut response) };
    LibError::check(return_code)?;

    let pacakge_diff_entries = unsafe { response.diff_packages.as_slice() };
    for entry in pacakge_diff_entries {
        let name = unsafe { entry.name.as_str() };
        let version = unsafe { entry.version.display() };

        let (marker, colored_name) = match entry.kind {
            CDiffKind::Added => ("+".green().bold(), name.green()),
            CDiffKind::Removed => ("-".red().bold(), name.red()),
            CDiffKind::Modified => ("~".yellow().bold(), name.yellow()),
        };
        println!("{} {} {}", marker, colored_name.bold(), version);
    }

    unsafe { (ctx.lib.free_response)(&mut response) };

    Ok(())
}
