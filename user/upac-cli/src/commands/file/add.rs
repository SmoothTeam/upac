// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use std::env::temp_dir;
use std::ffi::CString;
use std::ptr::null_mut;

use crate::cancel_token_ptr;
use crate::ffi::ctypes::{CDiffKind, CSlice};
use crate::ffi::packages::CPackageInfo;
use crate::ffi::request::CMutatedRequest;
use crate::types::errors::LibError;
use crate::types::CommandContext;

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
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let package_name = CString::new(args.package)?;
    let package_arch = CString::new(args.arch)?;
    let package_arch_sub = args.arch_sub.map(CString::new).transpose()?;

    let tmp_path = CString::new(
        temp_dir()
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("{}", gettextrs::gettext("err_tmp_path_encoding")))?,
    )?;

    let file_cstrings = args
        .files
        .iter()
        .map(|file_path| CString::new(file_path.as_str()))
        .collect::<Result<Vec<_>, _>>()?;

    let file_slices: Vec<CSlice> = file_cstrings.iter().map(CSlice::from_cstring).collect();

    let package_info = CPackageInfo::new(&package_name, &package_arch, package_arch_sub.as_ref());

    let request = CMutatedRequest::for_files(
        &file_slices,
        CDiffKind::Added,
        &package_info,
        &ctx.config.paths.repo_path,
        &ctx.config.paths.root_path,
        &tmp_path,
        &ctx.config.ostree.branch,
        None,
        null_mut(),
        cancel_token_ptr(),
    );

    let return_code = unsafe { (ctx.lib.file.files)(request) };
    LibError::check(return_code)?;

    Ok(())
}
