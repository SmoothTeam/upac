// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use chrono::{Local, TimeZone};

use clap::Args as ClapArgs;

use colored::Colorize;

use upac_types::request::unmutated::ListHistoryRequest;
use upac_types::response::entry::HistoryEntry;

use crate::types::{CommandContext, query, request_base};

#[derive(ClapArgs)]
pub struct Args {}

pub fn run(_args: Args, ctx: CommandContext) -> Result<()> {
    let request = ListHistoryRequest { base: request_base!() };

    let entries: Vec<HistoryEntry> = query!(ctx.lib.ro.list_history, request, history)?;
    for (index, entry) in entries.iter().enumerate() {
        let working_config = entry.working_config.as_deref();

        println!("{}", entry.subject.bold());
        if let Some(timestamp) = Local.timestamp_opt(entry.timestamp as i64, 0).single() {
            println!("{}", timestamp.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
        }
        println!("{}", entry.prefix_digest.yellow());

        for config in &entry.config_history {
            let marker = if working_config == Some(config.config_digest.as_str()) {
                "*"
            } else {
                " "
            };

            println!("  {marker} {} {}", config.subject, config.config_digest.yellow());
        }

        if index < entries.len() - 1 {
            println!();
        }
    }

    Ok(())
}
