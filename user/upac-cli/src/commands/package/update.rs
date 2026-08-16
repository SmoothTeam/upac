// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use std::ffi::CString;
use std::fs::canonicalize;

use upac_abi::request::CUpdateRequest;

use crate::types::CommandContext;
use crate::types::abi::{borrowed_vec, invoke, optional_slice, request_base, slice_from_cstr};

#[derive(clap::Args)]
pub struct Args {
    // Required flag, not a positional: keeps the positional slot free for a future
    // name-based network update (e.g. `up pkg update foo`), separate from this local-file path.
    #[arg(short, long = "file", required = true, num_args = 1..)]
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

    let request = CUpdateRequest::new(
        request_base(),
        slice_from_cstr(&ctx.tmp_path),
        slice_from_cstr(&subject),
        optional_slice(message.as_ref()),
        borrowed_vec(&path_slices),
    );

    invoke(|error| unsafe { (symbols.update)(request, error) })
}
