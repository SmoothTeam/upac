// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::CString;

use anyhow::Result;

use clap::Args as ClapArgs;

use colored::Colorize;

use upac_abi::request::CDiffRequest;
use upac_abi::{DiffFileSource, FileDiffKind, PackageDiffKind};

use crate::commands::display::VersionDisplay;
use crate::types::CommandContext;
use crate::types::abi::{invoke_with_response, optional_slice, request_base};

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long)]
    pub from_prefix: Option<String>,
    #[arg(long)]
    pub to_prefix: Option<String>,
    #[arg(long)]
    pub from_config: Option<String>,
    #[arg(long)]
    pub to_config: Option<String>,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let from_prefix = args.from_prefix.as_deref().map(CString::new).transpose()?;
    let to_prefix = args.to_prefix.as_deref().map(CString::new).transpose()?;
    let from_config = args.from_config.as_deref().map(CString::new).transpose()?;
    let to_config = args.to_config.as_deref().map(CString::new).transpose()?;

    let request = CDiffRequest::new(
        request_base(),
        optional_slice(from_prefix.as_ref()),
        optional_slice(to_prefix.as_ref()),
        optional_slice(from_config.as_ref()),
        optional_slice(to_config.as_ref()),
    );

    let response = invoke_with_response(|out, error| unsafe { (ctx.lib.ro.diff)(request, out, error) })?;

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

    for entry in unsafe { response.unattached_files.as_slice() } {
        let path = <&str>::try_from(&entry.common.path).unwrap_or_default();
        let source = match entry.source {
            DiffFileSource::Prefix => "prefix",
            DiffFileSource::Config => "config",
        };

        let (marker, colored_path) = match entry.common.kind {
            FileDiffKind::Added => ("+".green().bold(), path.green()),
            FileDiffKind::Removed => ("-".red().bold(), path.red()),
            FileDiffKind::Modified => ("~".yellow().bold(), path.yellow()),
        };

        println!("{} {} ({source})", marker, colored_path.bold());
    }

    unsafe { response.free() };

    Ok(())
}
