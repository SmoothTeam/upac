// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use std::mem::size_of;

use colored::Colorize;

use upac_abi::request::CListHistoryRequest;

use crate::types::CommandContext;
use crate::types::abi::{invoke_with_response, request_base};

#[derive(clap::Args)]
pub struct Args {}

pub fn run(_args: Args, ctx: CommandContext) -> Result<()> {
    let request = CListHistoryRequest {
        struct_size: size_of::<CListHistoryRequest>(),
        base: request_base(),
    };

    let response = invoke_with_response(|out, error| unsafe { (ctx.lib.ro.list_history)(request, out, error) })?;

    let commits = unsafe { response.history.as_slice() };
    for (index, commit) in commits.iter().enumerate() {
        let digest = unsafe { commit.prefix_digest.as_str() }.unwrap_or("");
        let subject = unsafe { commit.subject.as_str() }.unwrap_or("");

        println!("{}", subject.bold());
        println!("{}", digest.yellow());

        if index < commits.len() - 1 {
            println!();
        }
    }

    unsafe { response.free() };

    Ok(())
}
