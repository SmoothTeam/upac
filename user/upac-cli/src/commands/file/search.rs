// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::Args as ClapArgs;

use colored::Colorize;

use i18n_embed_fl::fl;

use upac_types::package::PackageInfo;
use upac_types::request::unmutated::{SearchFilesRequest, SearchInPackageFilesRequest};
use upac_types::response::entry::SearchFileEntry;

use crate::locale::LOADER;
use crate::types::{CommandContext, query, request_base};

#[derive(ClapArgs)]
pub struct Args {
    pub query: String,
    #[arg(long)]
    pub package: Option<String>,
    #[arg(long)]
    pub package_arch: Option<String>,
    #[arg(long)]
    pub package_arch_sub: Option<String>,
    #[arg(long)]
    pub regex: bool,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let entries = match args.package.as_deref() {
        Some(package) => search_in_package(&args, &ctx, package)?,
        None => search_all(&args, &ctx)?,
    };

    for entry in &entries {
        if entry.package_name.is_empty() {
            println!("{}", entry.path.bold());
        } else {
            println!("{} ({})", entry.path.bold(), entry.package_name);
        }
    }

    Ok(())
}

fn search_in_package(args: &Args, ctx: &CommandContext, package: &str) -> Result<Vec<SearchFileEntry>> {
    let Some(arch) = args.package_arch.as_deref() else {
        anyhow::bail!(fl!(LOADER, "err-invalid-entry"));
    };

    let request = SearchInPackageFilesRequest {
        base: request_base!(),
        package: PackageInfo {
            name: package.to_owned(),
            arch: arch.to_owned(),
            arch_sub: args.package_arch_sub.clone(),
        },
        search: &args.query,
        is_regex: args.regex,
    };

    query!(ctx.lib.ro.search_in_package_files, request, files)
}

fn search_all(args: &Args, ctx: &CommandContext) -> Result<Vec<SearchFileEntry>> {
    let request = SearchFilesRequest {
        base: request_base!(),
        search: &args.query,
        is_regex: args.regex,
    };

    query!(ctx.lib.ro.search_files, request, files)
}
