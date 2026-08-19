// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::CString;

use anyhow::Result;

use clap::Args as ClapArgs;

use colored::Colorize;

use upac_abi::PackageDiffKind;
use upac_abi::request::CDiffPackagesRequest;

use crate::commands::display::VersionDisplay;
use crate::types::CommandContext;
use crate::types::abi::{invoke_with_response, optional_slice, request_base};

#[derive(ClapArgs)]
pub struct Args {
    pub from: Option<String>,
    pub to: Option<String>,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let from_prefix = args.from.as_deref().map(CString::new).transpose()?;
    let to_prefix = args.to.as_deref().map(CString::new).transpose()?;

    let request = CDiffPackagesRequest::new(
        request_base(),
        optional_slice(from_prefix.as_ref()),
        optional_slice(to_prefix.as_ref()),
    );

    let response = invoke_with_response(|out, error| unsafe { (ctx.lib.ro.diff_packages)(request, out, error) })?;

    for entry in unsafe { response.diff_packages.as_slice() } {
        let name = <&str>::try_from(&entry.name).unwrap_or_default();
        let version = VersionDisplay(&entry.version);

        let (marker, colored_name) = match entry.kind {
            PackageDiffKind::Added => ("+".green().bold(), name.green()),
            PackageDiffKind::Removed => ("-".red().bold(), name.red()),
            PackageDiffKind::Modified => ("~".yellow().bold(), name.yellow()),
            PackageDiffKind::FilesChanged => ("*".yellow().bold(), name.yellow()),
        };
        println!("{} {} {}", marker, colored_name.bold(), version);
    }

    unsafe { response.free() };

    Ok(())
}
