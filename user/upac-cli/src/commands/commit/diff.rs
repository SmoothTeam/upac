// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::CString;

use anyhow::Result;

use clap::Args as ClapArgs;

use colored::Colorize;

use upac_abi::FileDiffKind;
use upac_abi::request::CDiffConfigRequest;

use crate::types::CommandContext;
use crate::types::abi::{invoke_with_response, optional_slice, request_base};

#[derive(ClapArgs)]
pub struct Args {
    pub from: Option<String>,
    pub to: Option<String>,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let from_config = args.from.as_deref().map(CString::new).transpose()?;
    let to_config = args.to.as_deref().map(CString::new).transpose()?;

    let request = CDiffConfigRequest::new(
        request_base(),
        optional_slice(from_config.as_ref()),
        optional_slice(to_config.as_ref()),
    );

    let response = invoke_with_response(|out, error| unsafe { (ctx.lib.ro.diff_config)(request, out, error) })?;

    for entry in unsafe { response.files.as_slice() } {
        let path = <&str>::try_from(&entry.common.path).unwrap_or_default();
        let package_name = Option::<&str>::try_from(&entry.package_name).unwrap_or_default();

        let (marker, colored_path) = match entry.common.kind {
            FileDiffKind::Added => ("+".green().bold(), path.green()),
            FileDiffKind::Removed => ("-".red().bold(), path.red()),
            FileDiffKind::Modified => ("~".yellow().bold(), path.yellow()),
        };

        match package_name {
            Some(package_name) => println!("{} {} ({package_name})", marker, colored_path.bold()),
            None => println!("{} {}", marker, colored_path.bold()),
        }
    }

    unsafe { response.free() };

    Ok(())
}
