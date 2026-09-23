// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ptr::null_mut;

use anyhow::Result;

use clap::Args as ClapArgs;

use i18n_embed_fl::fl;

use upac_types::package::{PackageInfo, PackageMeta};
use upac_types::request::RequestBase;
use upac_types::request::unmutated::{SearchInMetaRequest, SearchMetaRequest};

use crate::cancel_token_ptr;
use crate::commands::display::{PackageField, PackageFormatter};
use crate::locale::LOADER;
use crate::types::CommandContext;
use crate::types::abi::invoke_with_response;

#[cfg(test)]
#[path = "../../../tests/inline/search.rs"]
mod tests;

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
    pub version: bool,
    #[arg(long)]
    pub arch: bool,
    #[arg(long)]
    pub author: bool,
    #[arg(long)]
    pub license: bool,
    #[arg(long)]
    pub url: bool,
    #[arg(long)]
    pub packager: bool,
    #[arg(long)]
    pub size: bool,
    #[arg(long)]
    pub description: bool,
    #[arg(long)]
    pub checksum: bool,
    #[arg(long)]
    pub regex: bool,
    #[arg(long, value_enum)]
    pub sort: Option<PackageField>,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    match args.package.as_deref() {
        Some(package) => search_in_meta(&args, &ctx, package),
        None => search_meta(&args, &ctx),
    }
}

fn search_in_meta(args: &Args, ctx: &CommandContext, package: &str) -> Result<()> {
    let Some(arch) = args.package_arch.as_deref() else {
        anyhow::bail!(fl!(LOADER, "err-invalid-entry"));
    };

    let request = SearchInMetaRequest {
        base: RequestBase {
            on_hook: None,
            hook_ctx: null_mut(),
            cancel_token: cancel_token_ptr(),
        },
        package: PackageInfo {
            name: package.to_owned(),
            arch: arch.to_owned(),
            arch_sub: args.package_arch_sub.clone(),
        },
        search: &args.query,
        is_regex: args.regex,
    }
    .into();

    let response = invoke_with_response(|out, error| unsafe { (ctx.lib.ro.search_in_meta)(request, out, error) })?;

    let metas: Vec<PackageMeta> = Vec::try_from(&response.metas).unwrap_or_default();

    PackageFormatter {
        extra_fields: &build_extra_fields(args),
        metas: &metas,
        sort: args.sort,
    }
    .print();

    unsafe { response.free() };
    unsafe { request.free() };

    Ok(())
}

fn search_meta(args: &Args, ctx: &CommandContext) -> Result<()> {
    let request = SearchMetaRequest {
        base: RequestBase {
            on_hook: None,
            hook_ctx: null_mut(),
            cancel_token: cancel_token_ptr(),
        },
        search: &args.query,
        is_regex: args.regex,
    }
    .into();

    let response = invoke_with_response(|out, error| unsafe { (ctx.lib.ro.search_meta)(request, out, error) })?;

    let metas: Vec<PackageMeta> = Vec::try_from(&response.metas).unwrap_or_default();

    PackageFormatter {
        extra_fields: &build_extra_fields(args),
        metas: &metas,
        sort: args.sort,
    }
    .print();

    unsafe { response.free() };
    unsafe { request.free() };

    Ok(())
}

fn build_extra_fields(args: &Args) -> Vec<PackageField> {
    let mut fields = Vec::new();
    if args.version {
        fields.push(PackageField::Version);
    }
    if args.arch {
        fields.push(PackageField::Architecture);
    }
    if args.author {
        fields.push(PackageField::Author);
    }
    if args.license {
        fields.push(PackageField::License);
    }
    if args.url {
        fields.push(PackageField::Url);
    }
    if args.packager {
        fields.push(PackageField::Packager);
    }
    if args.size {
        fields.push(PackageField::Size);
    }
    if args.description {
        fields.push(PackageField::Description);
    }
    if args.checksum {
        fields.push(PackageField::Checksum);
    }
    fields
}
