// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::Args as ClapArgs;

use colored::Colorize;

use upac_abi::request::CListConfigRequest;

use crate::types::CommandContext;
use crate::types::abi::{empty_slice, invoke_with_response, request_base};

#[derive(ClapArgs)]
pub struct Args {}

pub fn run(_args: Args, ctx: CommandContext) -> Result<()> {
    let request = CListConfigRequest::new(request_base(), empty_slice());

    let response = invoke_with_response(|out, error| unsafe { (ctx.lib.ro.list_config)(request, out, error) })?;

    let commits = unsafe { response.commits.as_slice() };
    for (index, commit) in commits.iter().enumerate() {
        let digest = <&str>::try_from(&commit.config_digest).unwrap_or_default();
        let subject = <&str>::try_from(&commit.subject).unwrap_or_default();

        println!("{}", subject.bold());
        println!("{}", digest.yellow());

        if index < commits.len() - 1 {
            println!();
        }
    }

    unsafe { response.free() };

    Ok(())
}
