// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::CString;

use anyhow::Result;

use clap::Args as ClapArgs;

use colored::Colorize;

use upac_abi::request::{CSearchFilesRequest, CSearchInPackageFilesRequest};
use upac_abi::response::CSearchFileEntry;

use crate::types::CommandContext;
use crate::types::abi::{invoke_with_response, package_info, request_base, slice_from_cstr};

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
    let query = CString::new(args.query.as_str())?;

    match args.package.as_deref() {
        Some(package) => {
            let Some(arch) = args.package_arch.as_deref() else {
                anyhow::bail!(gettextrs::gettext("err_invalid_entry"));
            };

            let package_name = CString::new(package)?;
            let package_arch = CString::new(arch)?;
            let package_arch_sub = args.package_arch_sub.as_deref().map(CString::new).transpose()?;
            let package = package_info(&package_name, &package_arch, package_arch_sub.as_ref());

            let request =
                CSearchInPackageFilesRequest::new(request_base(), package, slice_from_cstr(&query), args.regex);
            let response = invoke_with_response(|out, error| unsafe {
                (ctx.lib.ro.search_in_package_files)(request, out, error)
            })?;

            print_entries(unsafe { response.files.as_slice() });

            unsafe { response.free() };
        }
        None => {
            let request = CSearchFilesRequest::new(request_base(), slice_from_cstr(&query), args.regex);
            let response =
                invoke_with_response(|out, error| unsafe { (ctx.lib.ro.search_files)(request, out, error) })?;

            print_entries(unsafe { response.files.as_slice() });

            unsafe { response.free() };
        }
    }

    Ok(())
}

fn print_entries(entries: &[CSearchFileEntry]) {
    for entry in entries {
        let path = <&str>::try_from(&entry.path).unwrap_or_default();
        let package_name = <&str>::try_from(&entry.package_name).unwrap_or_default();

        if package_name.is_empty() {
            println!("{}", path.bold());
        } else {
            println!("{} ({package_name})", path.bold());
        }
    }
}
