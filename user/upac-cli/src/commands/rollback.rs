// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ptr::null_mut;

use anyhow::Result;

use clap::Args as ClapArgs;

use i18n_embed_fl::fl;

use upac_types::request::RequestBase;
use upac_types::request::mutated::RollbackRequest;
use upac_types::settings::RuntimeSettings;

use crate::cancel_token_ptr;
use crate::locale::LOADER;
use crate::types::CommandContext;
use crate::types::abi::invoke;

#[derive(ClapArgs)]
pub struct Args {
    pub commit: String,
    #[arg(long)]
    pub boot: Option<String>,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let symbols = ctx.lib.require_write()?;

    let boot_plugin = args
        .boot
        .or_else(|| RuntimeSettings::load().boot.plugin)
        .ok_or_else(|| anyhow::anyhow!(fl!(LOADER, "err-boot-plugin-required")))?;

    let request = RollbackRequest {
        base: RequestBase {
            on_hook: None,
            hook_ctx: null_mut(),
            cancel_token: cancel_token_ptr(),
        },
        tmp_path: &ctx.tmp_path.to_string_lossy(),
        config_digest: &args.commit,
        boot_plugin: &boot_plugin,
    }
    .into();

    let result = invoke(|error| unsafe { (symbols.rollback)(request, error) });
    unsafe { request.free() };

    result
}
