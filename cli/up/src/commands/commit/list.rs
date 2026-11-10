// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::Args as ClapArgs;

use colored::Colorize;

use upac_types::request::unmutated::ListConfigRequest;
use upac_types::response::entry::ConfigCommitEntry;

use crate::types::{CommandContext, query, request_base};

#[derive(ClapArgs)]
pub struct Args {}

pub fn run(_args: Args, ctx: CommandContext) -> Result<()> {
    let request = ListConfigRequest {
        base: request_base!(),
        prefix_digest: None,
    };

    let commits: Vec<ConfigCommitEntry> = query!(ctx.lib.ro.list_config, request, commits)?;
    for (index, commit) in commits.iter().enumerate() {
        println!("{}", commit.subject.bold());
        println!("{}", commit.config_digest.yellow());

        if index < commits.len() - 1 {
            println!();
        }
    }

    Ok(())
}
