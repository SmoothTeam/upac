// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::CString;

use anyhow::Result;

use clap::Args as ClapArgs;

use upac_abi::request::{CSearchInMetaRequest, CSearchMetaRequest};

use crate::commands::display::{PackageField, PackageFormatter};
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
    let query = CString::new(args.query.as_str())?;
    let extra_fields = build_extra_fields(&args);

    match args.package.as_deref() {
        Some(package) => {
            let Some(arch) = args.package_arch.as_deref() else {
                anyhow::bail!(gettextrs::gettext("err_invalid_entry"));
            };

            let package_name = CString::new(package)?;
            let package_arch = CString::new(arch)?;
            let package_arch_sub = args.package_arch_sub.as_deref().map(CString::new).transpose()?;
            let package = package_info(&package_name, &package_arch, package_arch_sub.as_ref());

            let request = CSearchInMetaRequest::new(request_base(), package, slice_from_cstr(&query), args.regex);
            let response =
                invoke_with_response(|out, error| unsafe { (ctx.lib.ro.search_in_meta)(request, out, error) })?;

            PackageFormatter {
                extra_fields: &extra_fields,
                metas: unsafe { response.metas.as_slice() },
                sort: args.sort,
            }
            .print();

            unsafe { response.free() };
        }
        None => {
            let request = CSearchMetaRequest::new(request_base(), slice_from_cstr(&query), args.regex);
            let response = invoke_with_response(|out, error| unsafe { (ctx.lib.ro.search_meta)(request, out, error) })?;

            PackageFormatter {
                extra_fields: &extra_fields,
                metas: unsafe { response.metas.as_slice() },
                sort: args.sort,
            }
            .print();

            unsafe { response.free() };
        }
    }

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
