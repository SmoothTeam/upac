// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ptr::null_mut;

use anyhow::Result;

use chrono::{Local, TimeZone};

use clap::Args as ClapArgs;

use colored::Colorize;

use upac_types::request::RequestBase;
use upac_types::request::unmutated::ListHistoryRequest;
use upac_types::response::entry::HistoryEntry;

use crate::cancel_token_ptr;
use crate::types::CommandContext;
use crate::types::abi::invoke_with_response;

#[derive(ClapArgs)]
pub struct Args {}

pub fn run(_args: Args, ctx: CommandContext) -> Result<()> {
    let request = ListHistoryRequest {
        base: RequestBase {
            on_hook: None,
            hook_ctx: null_mut(),
            cancel_token: cancel_token_ptr(),
        },
    }
    .into();

    let response = invoke_with_response(|out, error| unsafe { (ctx.lib.ro.list_history)(request, out, error) })?;

    let entries: Vec<HistoryEntry> = Vec::try_from(&response.history).unwrap_or_default();
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

    unsafe { response.free() };
    unsafe { request.free() };

    Ok(())
}
