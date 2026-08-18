// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::CString;
use std::fs::canonicalize;

use anyhow::Result;

use clap::Args as ClapArgs;

use upac_abi::request::CInstallRequest;

use crate::types::CommandContext;
use crate::types::abi::{BootKind, borrowed_vec, invoke, optional_slice, request_base, slice_from_cstr};

#[derive(ClapArgs)]
pub struct Args {
    // Required flag, not a positional: keeps the positional slot free for a future
    // name-based network install (e.g. `up pkg install foo`), separate from this local-file path.
    #[arg(short, long = "file", required = true, num_args = 1..)]
    pub files: Vec<String>,
    #[arg(short, long)]
    pub message: Option<String>,
    #[arg(long, value_enum, default_value_t = BootKind::Auto)]
    pub boot: BootKind,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let symbols = ctx.lib.require_write()?;

    let subject = CString::new("install")?;
    let message = args.message.map(CString::new).transpose()?;

    let mut paths = Vec::with_capacity(args.files.len());
    for file_path in &args.files {
        let absolute = canonicalize(file_path)
            .map_err(|_| anyhow::anyhow!("{}: {file_path}", gettextrs::gettext("err_not_found")))?;
        paths.push(CString::new(absolute.to_string_lossy().as_ref())?);
    }

    let path_slices: Vec<_> = paths.iter().map(slice_from_cstr).collect();

    let request = CInstallRequest::new(
        request_base(),
        slice_from_cstr(&ctx.tmp_path),
        slice_from_cstr(&subject),
        optional_slice(message.as_ref()),
        borrowed_vec(&path_slices),
        args.boot.into(),
    );

    invoke(|error| unsafe { (symbols.install)(request, error) })
}
