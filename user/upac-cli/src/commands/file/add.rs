// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use std::ffi::CString;
use std::mem::size_of;

use upac_abi::FileDiffKind;
use upac_abi::request::CFilesRequest;

use crate::types::CommandContext;
use crate::types::abi::{borrowed_vec, invoke, optional_slice, package_info, request_base, slice_from_cstr};

#[derive(clap::Args)]
pub struct Args {
    #[arg(required = true, num_args = 1..)]
    pub files: Vec<String>,
    #[arg(long, required = true)]
    pub package: String,
    #[arg(long, required = true)]
    pub arch: String,
    #[arg(long)]
    pub arch_sub: Option<String>,
    #[arg(short, long)]
    pub message: Option<String>,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let symbols = ctx.lib.require_write()?;

    let package_name = CString::new(args.package)?;
    let package_arch = CString::new(args.arch)?;
    let package_arch_sub = args.arch_sub.map(CString::new).transpose()?;
    let subject = CString::new("file add")?;
    let message = args.message.map(CString::new).transpose()?;

    let file_cstrings = args
        .files
        .iter()
        .map(|file_path| CString::new(file_path.as_str()))
        .collect::<Result<Vec<_>, _>>()?;
    let file_slices: Vec<_> = file_cstrings.iter().map(slice_from_cstr).collect();

    let package = package_info(&package_name, &package_arch, package_arch_sub.as_ref());

    let request = CFilesRequest {
        struct_size: size_of::<CFilesRequest>(),
        base: request_base(),
        tmp_path: slice_from_cstr(&ctx.tmp_path),
        subject: slice_from_cstr(&subject),
        message: optional_slice(message.as_ref()),
        files: borrowed_vec(&file_slices),
        file_kind: FileDiffKind::Added,
        file_package: &package,
    };

    invoke(|error| unsafe { (symbols.files)(request, error) })
}
