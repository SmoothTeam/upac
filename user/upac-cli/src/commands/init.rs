// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use std::path::Path;

use gettextrs::gettext;

use crate::cancel_token_ptr;
use crate::config::Config;
use crate::ffi::ctypes::CSlice;
use crate::ffi::request::CUnmutatedRequest;
use crate::types::CommandContext;
use crate::types::errors::LibError;

#[derive(clap::Args)]
pub struct InitArgs {
    #[arg(long)]
    pub config_path: Option<String>,
}

pub fn run(args: InitArgs, ctx: CommandContext) -> Result<()> {
    let config = match args.config_path {
        Some(ref path) => Config::load(Path::new(path))?,
        None => ctx.config,
    };

    let mode_str = config
        .ostree
        .mode
        .to_str()
        .map_err(|_| anyhow::anyhow!("{}", gettext("err_config_mode_invalid")))?;

    let repo_mode: u32 = match mode_str {
        "archive" => 0,
        "bare" => 1,
        "bare-user" => 2,
        _ => anyhow::bail!("{}", gettext("err_config_mode_invalid")),
    };

    let symlink_slices: Vec<CSlice> = config.ostree.symlinks.iter().map(CSlice::from_cstring).collect();

    let request = CUnmutatedRequest::for_init(
        &config.paths.repo_path,
        &config.paths.root_path,
        &config.ostree.branch,
        &symlink_slices,
        &repo_mode,
        cancel_token_ptr(),
    );

    let return_code = unsafe { (ctx.lib.init)(request) };
    LibError::check(return_code)?;

    Ok(())
}
