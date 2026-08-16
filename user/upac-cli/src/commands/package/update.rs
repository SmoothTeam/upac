// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use std::ffi::CString;
use std::fs::canonicalize;
use std::mem::size_of;

use upac_abi::request::CUpdateRequest;

use crate::types::CommandContext;
use crate::types::abi::{borrowed_vec, invoke, optional_slice, request_base, slice_from_cstr};

#[derive(clap::Args)]
pub struct Args {
    #[arg(required = true, num_args = 1..)]
    pub files: Vec<String>,
    #[arg(short, long)]
    pub message: Option<String>,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let symbols = ctx.lib.require_write()?;

    let subject = CString::new("update")?;
    let message = args.message.map(CString::new).transpose()?;

    let mut paths = Vec::with_capacity(args.files.len());
    for file_path in &args.files {
        let absolute = canonicalize(file_path)
            .map_err(|_| anyhow::anyhow!("{}: {file_path}", gettextrs::gettext("err_not_found")))?;
        paths.push(CString::new(absolute.to_string_lossy().as_ref())?);
    }

    let path_slices: Vec<_> = paths.iter().map(slice_from_cstr).collect();

    let request = CUpdateRequest {
        struct_size: size_of::<CUpdateRequest>(),
        base: request_base(),
        tmp_path: slice_from_cstr(&ctx.tmp_path),
        subject: slice_from_cstr(&subject),
        message: optional_slice(message.as_ref()),
        packages: borrowed_vec(&path_slices),
    };

    invoke(|error| unsafe { (symbols.update)(request, error) })
}
