// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::CString;

use anyhow::Result;

use clap::Args as ClapArgs;

use upac_abi::FileDiffKind;
use upac_abi::error::ErrorDomain;
use upac_abi::request::{CFilesRequest, CRequestBase};

use crate::cancel_token_ptr;
use crate::types::CommandContext;
use crate::types::abi::{FileScope, borrowed_vec, invoke, optional_slice, package_info, slice_from_cstr};
use crate::types::progress::{ProgressState, on_progress};

#[derive(ClapArgs)]
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
    #[arg(long)]
    pub boot: Option<String>,
    #[arg(long, value_enum, default_value_t = FileScope::Usr)]
    pub scope: FileScope,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let symbols = ctx.lib.require_write()?;

    let package_name = CString::new(args.package)?;
    let package_arch = CString::new(args.arch)?;
    let package_arch_sub = args.arch_sub.map(CString::new).transpose()?;
    let subject = CString::new("file remove")?;
    let message = args.message.map(CString::new).transpose()?;
    let boot_plugin = args.boot.map(CString::new).transpose()?;
    let scope = args.scope.into();

    let file_cstrings = args
        .files
        .iter()
        .map(|file_path| CString::new(file_path.as_str()))
        .collect::<Result<Vec<_>, _>>()?;
    let file_slices: Vec<_> = file_cstrings.iter().map(slice_from_cstr).collect();

    let package = package_info(&package_name, &package_arch, package_arch_sub.as_ref());

    let mut progress = ProgressState::new(ErrorDomain::Files);
    let base = CRequestBase::new(Some(on_progress), progress.ctx_ptr(), cancel_token_ptr());

    let request = CFilesRequest::new(
        base,
        slice_from_cstr(&ctx.tmp_path),
        slice_from_cstr(&subject),
        optional_slice(message.as_ref()),
        borrowed_vec(&file_slices),
        FileDiffKind::Removed,
        scope,
        &package,
        optional_slice(boot_plugin.as_ref()),
    );

    let result = invoke(|error| unsafe { (symbols.files)(request, error) });
    progress.finish();

    result
}
