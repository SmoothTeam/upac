// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::Args as ClapArgs;

use i18n_embed_fl::fl;

use upac_types::package::{PackageInfo, PackageMeta};
use upac_types::request::unmutated::{SearchInMetaRequest, SearchMetaRequest};

use crate::commands::display::DisplayPakcageMetaArgs;
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
    #[command(flatten)]
    pub display_package_meta: DisplayPakcageMetaArgs,
    #[arg(long)]
    pub regex: bool,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let metas = match args.package.as_deref() {
        Some(package) => search_in_meta(&args, &ctx, package)?,
        None => search_meta(&args, &ctx)?,
    };

    args.display_package_meta.print(&metas);

    Ok(())
}

fn search_in_meta(args: &Args, ctx: &CommandContext, package: &str) -> Result<Vec<PackageMeta>> {
    let Some(arch) = args.package_arch.as_deref() else {
        anyhow::bail!(fl!(LOADER, "err-invalid-entry"));
    };

    let request = SearchInMetaRequest {
        base: request_base!(),
        package: PackageInfo {
            name: package.to_owned(),
            arch: arch.to_owned(),
            arch_sub: args.package_arch_sub.clone(),
        },
        search: &args.query,
        is_regex: args.regex,
    };

    query!(ctx.lib.ro.search_in_meta, request, metas)
}

fn search_meta(args: &Args, ctx: &CommandContext) -> Result<Vec<PackageMeta>> {
    let request = SearchMetaRequest {
        base: request_base!(),
        search: &args.query,
        is_regex: args.regex,
    };

    query!(ctx.lib.ro.search_meta, request, metas)
}
