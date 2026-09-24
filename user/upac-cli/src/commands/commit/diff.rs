// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::Args as ClapArgs;

use colored::Colorize;

use upac_abi::FileDiffKind;

use upac_types::request::unmutated::DiffConfigRequest;
use upac_types::response::entry::DiffConfigFileEntry;

use crate::types::{CommandContext, query, request_base};

#[derive(ClapArgs)]
pub struct Args {
    pub from: Option<String>,
    pub to: Option<String>,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let request = DiffConfigRequest {
        base: request_base!(),
        from_config_digest: args.from.as_deref(),
        to_config_digest: args.to.as_deref(),
    };

    let files: Vec<DiffConfigFileEntry> = query!(ctx.lib.ro.diff_config, request, files)?;

    for entry in &files {
        let path = entry.common.path.as_str();

        let (marker, colored_path) = match entry.common.kind {
            FileDiffKind::Added => ("+".green().bold(), path.green()),
            FileDiffKind::Removed => ("-".red().bold(), path.red()),
            FileDiffKind::Modified => ("~".yellow().bold(), path.yellow()),
        };

        match entry.package_name.as_deref() {
            Some(package_name) => println!("{} {} ({package_name})", marker, colored_path.bold()),
            None => println!("{} {}", marker, colored_path.bold()),
        }
    }

    Ok(())
}
