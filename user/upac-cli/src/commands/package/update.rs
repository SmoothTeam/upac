// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::fs::canonicalize;

use anyhow::Result;

use clap::Args as ClapArgs;

use i18n_embed_fl::fl;

use upac_abi::error::ErrorDomain;

use upac_types::request::mutated::UpdateRequest;

use crate::locale::{LOADER, SUBJECT_LOADER};
use crate::types::progress::ProgressState;
use crate::types::{CommandContext, boot_plugin, call, request_base};

#[derive(ClapArgs)]
pub struct Args {
    #[arg(short, long = "file", required = true, num_args = 1..)]
    pub files: Vec<String>,
    #[arg(short, long)]
    pub message: Option<String>,
    #[arg(long)]
    pub boot: Option<String>,
    #[arg(long)]
    pub allow_downgrade: bool,
    #[arg(long)]
    pub no_conflict_files: bool,
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let symbols = ctx.lib.require_write()?;

    let mut packages = Vec::with_capacity(args.files.len());
    for file_path in &args.files {
        let absolute =
            canonicalize(file_path).map_err(|_| anyhow::anyhow!("{}: {file_path}", fl!(LOADER, "err-not-found")))?;
        packages.push(absolute.to_string_lossy().into_owned());
    }

    let mut progress = ProgressState::new(ErrorDomain::Update);

    let boot_plugin = boot_plugin!(args.boot)?;

    let subject = fl!(SUBJECT_LOADER, "subject-update");

    let request = UpdateRequest {
        base: request_base!(progress),
        tmp_path: &ctx.tmp_path,
        subject: &subject,
        message: args.message.as_deref(),
        packages: packages.iter().map(|string| string.as_str()).collect(),
        boot_plugin: &boot_plugin,
        allow_downgrade: args.allow_downgrade,
        allow_conflict_files: !args.no_conflict_files,
    };

    call!(symbols.update, request)
}
