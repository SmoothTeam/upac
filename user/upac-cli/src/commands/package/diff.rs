// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::Args as ClapArgs;

use colored::Colorize;

use upac_abi::PackageDiffKind;

use upac_types::request::unmutated::DiffPackagesRequest;
use upac_types::response::entry::DiffPackageEntry;

use crate::commands::display::VersionDisplay;
use crate::types::{CommandContext, query, request_base};

#[derive(ClapArgs)]
pub struct Args {
    pub from: Option<String>,
    pub to: Option<String>,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let request = DiffPackagesRequest {
        base: request_base!(),
        from_prefix_digest: args.from.as_deref(),
        to_prefix_digest: args.to.as_deref(),
    };

    let diff_packages: Vec<DiffPackageEntry> = query!(ctx.lib.ro.diff_packages, request, diff_packages)?;

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

    Ok(())
}
