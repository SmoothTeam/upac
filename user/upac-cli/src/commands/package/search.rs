// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use std::ffi::CString;

use crate::cancel_token_ptr;
use crate::ffi::request::{CUnmutatedRequest, CUnmutatedResponse};
use crate::types::CommandContext;
use crate::types::errors::LibError;
use crate::types::package::{PackageField, PackageFormatter};

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
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let query = CString::new(args.query.as_str())?;
    let mut response = CUnmutatedResponse::empty();

    let request = CUnmutatedRequest::for_search(&ctx.config.paths.root_path, &query, cancel_token_ptr());

    let return_code = unsafe { (ctx.lib.pkg.search)(request, &mut response) };
    LibError::check(return_code)?;

    let metas = unsafe { response.metas.as_slice() };

    let extra_fields = build_extra_fields(&args);
    PackageFormatter {
        extra_fields: &extra_fields,
        metas,
    }
    .print();

    unsafe { (ctx.lib.free_response)(&mut response) };

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
