// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::CString;
use std::fs::canonicalize;

use anyhow::Result;

use clap::Args as ClapArgs;

use i18n_embed_fl::fl;

use upac_abi::error::ErrorDomain;
use upac_abi::request::{CInstallRequest, CRequestBase};

use crate::cancel_token_ptr;
use crate::locale::LOADER;
use crate::types::CommandContext;
use crate::types::abi::{borrowed_vec, invoke, optional_slice, slice_from_cstr};
use crate::types::progress::{ProgressState, on_progress};

#[derive(ClapArgs)]
pub struct Args {
    // Required flag, not a positional: keeps the positional slot free for a future
    // name-based network install (e.g. `up pkg install foo`), separate from this local-file path.
    #[arg(short, long = "file", required = true, num_args = 1..)]
    pub files: Vec<String>,
    #[arg(short, long)]
    pub message: Option<String>,
    #[arg(long)]
    pub boot: Option<String>,
    #[arg(long)]
    pub no_conflict_files: bool,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let symbols = ctx.lib.require_write()?;

    let subject = CString::new("install")?;
    let message = args.message.map(CString::new).transpose()?;
    let boot_plugin = args.boot.map(CString::new).transpose()?;

    let mut paths = Vec::with_capacity(args.files.len());
    for file_path in &args.files {
        let absolute =
            canonicalize(file_path).map_err(|_| anyhow::anyhow!("{}: {file_path}", fl!(LOADER, "err-not-found")))?;
        paths.push(CString::new(absolute.to_string_lossy().as_ref())?);
    }

    let path_slices: Vec<_> = paths.iter().map(slice_from_cstr).collect();

    let mut progress = ProgressState::new(ErrorDomain::Install);
    let base = CRequestBase::new(Some(on_progress), progress.ctx_ptr(), cancel_token_ptr());

    let request = CInstallRequest::new(
        base,
        slice_from_cstr(&ctx.tmp_path),
        slice_from_cstr(&subject),
        optional_slice(message.as_ref()),
        borrowed_vec(&path_slices),
        optional_slice(boot_plugin.as_ref()),
        !args.no_conflict_files,
    );

    let result = invoke(|error| unsafe { (symbols.install)(request, error) });
    progress.finish();

    result
}
