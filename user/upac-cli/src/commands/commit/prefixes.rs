// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use chrono::{Local, TimeZone};

use clap::Args as ClapArgs;

use colored::Colorize;

use upac_abi::request::CListPrefixRequest;

use crate::types::CommandContext;
use crate::types::abi::{invoke_with_response, request_base};

#[derive(ClapArgs)]
pub struct Args {}

pub fn run(_args: Args, ctx: CommandContext) -> Result<()> {
    let request = CListPrefixRequest::new(request_base());

    let response = invoke_with_response(|out, error| unsafe { (ctx.lib.ro.list_prefix)(request, out, error) })?;

    let prefixes = unsafe { response.prefixes.as_slice() };
    for (index, prefix) in prefixes.iter().enumerate() {
        let digest = <&str>::try_from(&prefix.prefix_digest).unwrap_or_default();
        let subject = <&str>::try_from(&prefix.subject).unwrap_or_default();

        println!("{}", subject.bold());
        if let Some(timestamp) = Local.timestamp_opt(prefix.timestamp as i64, 0).single() {
            println!("{}", timestamp.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
        }
        println!("{}", digest.yellow());

        if index < prefixes.len() - 1 {
            println!();
        }
    }

    unsafe { response.free() };

    Ok(())
}
