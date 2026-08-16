// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use std::ffi::CString;

use upac_abi::request::CSearchMetaRequest;

use crate::commands::display::package::{PackageField, PackageFormatter};
use crate::types::CommandContext;
use crate::types::abi::{invoke_with_response, request_base, slice_from_cstr};

#[derive(clap::Args)]
pub struct Args {
    pub query: String,
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
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let query = CString::new(args.query.as_str())?;

    let request = CSearchMetaRequest::new(request_base(), slice_from_cstr(&query), args.regex);

    let response = invoke_with_response(|out, error| unsafe { (ctx.lib.ro.search_meta)(request, out, error) })?;

    let extra_fields = build_extra_fields(&args);
    PackageFormatter {
        extra_fields: &extra_fields,
        metas: unsafe { response.metas.as_slice() },
    }
    .print();

    unsafe { response.free() };

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
