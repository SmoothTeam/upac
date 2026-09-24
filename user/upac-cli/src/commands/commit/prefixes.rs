// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use chrono::{Local, TimeZone};

use clap::Args as ClapArgs;

use colored::Colorize;

use upac_types::request::unmutated::ListPrefixRequest;
use upac_types::response::entry::PrefixEntry;

use crate::types::{CommandContext, query, request_base};

#[derive(ClapArgs)]
pub struct Args {}

pub fn run(_args: Args, ctx: CommandContext) -> Result<()> {
    let request = ListPrefixRequest { base: request_base!() };

    let prefixes: Vec<PrefixEntry> = query!(ctx.lib.ro.list_prefix, request, prefixes)?;

    for (index, prefix) in prefixes.iter().enumerate() {
        println!("{}", prefix.subject.bold());
        if let Some(timestamp) = Local.timestamp_opt(prefix.timestamp as i64, 0).single() {
            println!("{}", timestamp.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
        }
        println!("{}", prefix.prefix_digest.yellow());

        if index < prefixes.len() - 1 {
            println!();
        }
    }

    Ok(())
}
