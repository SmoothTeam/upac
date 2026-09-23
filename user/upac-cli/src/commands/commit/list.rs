// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ptr::null_mut;

use anyhow::Result;

use clap::Args as ClapArgs;

use colored::Colorize;

use upac_types::request::RequestBase;
use upac_types::request::unmutated::ListConfigRequest;
use upac_types::response::entry::ConfigCommitEntry;

use crate::cancel_token_ptr;
use crate::types::CommandContext;
use crate::types::abi::invoke_with_response;

#[derive(ClapArgs)]
pub struct Args {}

pub fn run(_args: Args, ctx: CommandContext) -> Result<()> {
    let request = ListConfigRequest {
        base: RequestBase {
            on_hook: None,
            hook_ctx: null_mut(),
            cancel_token: cancel_token_ptr(),
        },
        prefix_digest: None,
    }
    .into();

    let response = invoke_with_response(|out, error| unsafe { (ctx.lib.ro.list_config)(request, out, error) })?;

    let commits: Vec<ConfigCommitEntry> = Vec::try_from(&response.commits).unwrap_or_default();
    for (index, commit) in commits.iter().enumerate() {
        println!("{}", commit.subject.bold());
        println!("{}", commit.config_digest.yellow());

        if index < commits.len() - 1 {
            println!();
        }
    }

    unsafe { response.free() };
    unsafe { request.free() };

    Ok(())
}
