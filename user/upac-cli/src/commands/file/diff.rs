// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::Args as ClapArgs;

use colored::Colorize;

use upac_abi::FileDiffKind;

use upac_types::request::unmutated::DiffPrefixRequest;
use upac_types::response::entry::DiffPrefixFileEntry;

use crate::types::{CommandContext, query, request_base};

#[derive(ClapArgs)]
pub struct Args {
    pub from: Option<String>,
    pub to: Option<String>,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let request = DiffPrefixRequest {
        base: request_base!(),
        from_prefix_digest: args.from.as_deref(),
        to_prefix_digest: args.to.as_deref(),
    };

    let files: Vec<DiffPrefixFileEntry> = query!(ctx.lib.ro.diff_prefix, request, files)?;

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

    Ok(())
}
