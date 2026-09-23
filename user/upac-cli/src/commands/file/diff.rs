// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ptr::null_mut;

use anyhow::Result;

use clap::Args as ClapArgs;

use colored::Colorize;

use upac_abi::FileDiffKind;

use upac_types::request::RequestBase;
use upac_types::request::unmutated::DiffPrefixRequest;
use upac_types::response::entry::DiffPrefixFileEntry;

use crate::cancel_token_ptr;
use crate::types::CommandContext;
use crate::types::abi::invoke_with_response;

#[derive(ClapArgs)]
pub struct Args {
    pub from: Option<String>,
    pub to: Option<String>,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let request = DiffPrefixRequest {
        base: RequestBase {
            on_hook: None,
            hook_ctx: null_mut(),
            cancel_token: cancel_token_ptr(),
        },
        from_prefix_digest: args.from.as_deref(),
        to_prefix_digest: args.to.as_deref(),
    }
    .into();

    let response = invoke_with_response(|out, error| unsafe { (ctx.lib.ro.diff_prefix)(request, out, error) })?;

    let files: Vec<DiffPrefixFileEntry> = Vec::try_from(&response.files).unwrap_or_default();

    for entry in &files {
        let path = entry.common.path.as_str();

        let (marker, colored_path) = match entry.common.kind {
            FileDiffKind::Added => ("+".green().bold(), path.green()),
            FileDiffKind::Removed => ("-".red().bold(), path.red()),
            FileDiffKind::Modified => ("~".yellow().bold(), path.yellow()),
        };

        if entry.package_name.is_empty() {
            println!("{} {}", marker, colored_path.bold());
        } else {
            println!("{} {} ({})", marker, colored_path.bold(), entry.package_name);
        }
    }

    unsafe { response.free() };
    unsafe { request.free() };

    Ok(())
}
