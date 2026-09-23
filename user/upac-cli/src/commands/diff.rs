// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ptr::null_mut;

use anyhow::Result;

use clap::Args as ClapArgs;

use colored::Colorize;

use upac_abi::{FileDiffKind, PackageDiffKind};

use upac_types::request::RequestBase;
use upac_types::request::unmutated::DiffRequest;
use upac_types::response::entry::{DiffPackageEntry, DiffUntrackedFileEntry};

use crate::cancel_token_ptr;
use crate::commands::display::VersionDisplay;
use crate::types::CommandContext;
use crate::types::abi::invoke_with_response;

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long)]
    pub from_prefix: Option<String>,
    #[arg(long)]
    pub to_prefix: Option<String>,
    #[arg(long)]
    pub from_config: Option<String>,
    #[arg(long)]
    pub to_config: Option<String>,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let request = DiffRequest {
        base: RequestBase {
            on_hook: None,
            hook_ctx: null_mut(),
            cancel_token: cancel_token_ptr(),
        },
        from_prefix_digest: args.from_prefix.as_deref(),
        to_prefix_digest: args.to_prefix.as_deref(),
        from_config_digest: args.from_config.as_deref(),
        to_config_digest: args.to_config.as_deref(),
    }
    .into();

    let response = invoke_with_response(|out, error| unsafe { (ctx.lib.ro.diff)(request, out, error) })?;

    let diff_packages: Vec<DiffPackageEntry> = Vec::try_from(&response.diff_packages).unwrap_or_default();
    let unattached_files: Vec<DiffUntrackedFileEntry> = Vec::try_from(&response.unattached_files).unwrap_or_default();

    for entry in &diff_packages {
        let name = entry.name.as_str();

        let (marker, colored_name) = match entry.kind {
            PackageDiffKind::Added => ("+".green().bold(), name.green()),
            PackageDiffKind::Removed => ("-".red().bold(), name.red()),
            PackageDiffKind::Modified => ("~".yellow().bold(), name.yellow()),
            PackageDiffKind::FilesChanged => ("*".yellow().bold(), name.yellow()),
        };
        println!("{} {} {}", marker, colored_name.bold(), VersionDisplay(&entry.version));
    }

    for entry in &unattached_files {
        let path = entry.common.path.as_str();

        let (marker, colored_path) = match entry.common.kind {
            FileDiffKind::Added => ("+".green().bold(), path.green()),
            FileDiffKind::Removed => ("-".red().bold(), path.red()),
            FileDiffKind::Modified => ("~".yellow().bold(), path.yellow()),
        };

        println!("{} {} ({})", marker, colored_path.bold(), entry.source.as_str());
    }

    unsafe { response.free() };
    unsafe { request.free() };

    Ok(())
}
