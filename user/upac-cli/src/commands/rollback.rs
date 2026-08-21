// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::CString;

use anyhow::Result;

use clap::Args as ClapArgs;

use upac_abi::request::CRollbackRequest;

use crate::types::CommandContext;
use crate::types::abi::{BootKind, invoke, request_base, slice_from_cstr};

#[derive(ClapArgs)]
pub struct Args {
    pub commit: String,
    #[arg(long, value_enum, default_value_t = BootKind::Auto)]
    pub boot: BootKind,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let symbols = ctx.lib.require_write()?;
    let config_digest = CString::new(args.commit)?;

    let request = CRollbackRequest::new(
        request_base(),
        slice_from_cstr(&ctx.tmp_path),
        slice_from_cstr(&config_digest),
        args.boot.into(),
    );

    invoke(|error| unsafe { (symbols.rollback)(request, error) })
}
