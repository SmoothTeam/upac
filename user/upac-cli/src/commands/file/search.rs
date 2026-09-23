// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ptr::null_mut;

use anyhow::Result;

use clap::Args as ClapArgs;

use colored::Colorize;

use i18n_embed_fl::fl;

use upac_types::package::PackageInfo;
use upac_types::request::RequestBase;
use upac_types::request::unmutated::{SearchFilesRequest, SearchInPackageFilesRequest};

use crate::cancel_token_ptr;
use crate::locale::LOADER;
use crate::types::CommandContext;
use crate::types::abi::invoke_with_response;

macro_rules! print_entries {
    ($entries:expr) => {
        for entry in $entries {
            let path = <&str>::try_from(&entry.path).unwrap_or_default();
            let package_name = <&str>::try_from(&entry.package_name).unwrap_or_default();

            if package_name.is_empty() {
                println!("{}", path.bold());
            } else {
                println!("{} ({package_name})", path.bold());
            }
        }
    };
}

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
    match args.package.as_deref() {
        Some(package) => search_in_package(&args, ctx, package),
        None => search_all(&args, ctx),
    }
}

fn search_in_package(args: &Args, ctx: CommandContext, package: &str) -> Result<()> {
    let Some(arch) = args.package_arch.as_deref() else {
        anyhow::bail!(fl!(LOADER, "err-invalid-entry"));
    };

    let request = SearchInPackageFilesRequest {
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

    let response =
        invoke_with_response(|out, error| unsafe { (ctx.lib.ro.search_in_package_files)(request, out, error) })?;

    print_entries!(unsafe { response.files.as_slice() });

    unsafe { response.free() };
    unsafe { request.free() };

    Ok(())
}

fn search_all(args: &Args, ctx: CommandContext) -> Result<()> {
    let request = SearchFilesRequest {
        base: RequestBase {
            on_hook: None,
            hook_ctx: null_mut(),
            cancel_token: cancel_token_ptr(),
        },
        search: &args.query,
        is_regex: args.regex,
    }
    .into();

    let response = invoke_with_response(|out, error| unsafe { (ctx.lib.ro.search_files)(request, out, error) })?;

    print_entries!(unsafe { response.files.as_slice() });

    unsafe { response.free() };
    unsafe { request.free() };

    Ok(())
}
