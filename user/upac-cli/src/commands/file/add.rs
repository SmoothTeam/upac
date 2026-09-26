// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::Args as ClapArgs;

use i18n_embed_fl::fl;

use upac_abi::error::ErrorDomain;
use upac_abi::response::entry::FileDiffKind;

use upac_types::package::PackageInfo;
use upac_types::request::mutated::FilesRequest;

use crate::locale::SUBJECT_LOADER;
use crate::types::abi::FileScope;
use crate::types::progress::ProgressState;
use crate::types::{CommandContext, boot_plugin, call, request_base};

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

    let package = PackageInfo {
        name: args.package,
        arch: args.arch,
        arch_sub: args.arch_sub,
    }
    .into();

    let mut progress = ProgressState::new(ErrorDomain::Files);

    let boot_plugin = boot_plugin!(args.boot)?;

    let subject = fl!(SUBJECT_LOADER, "subject-file-add");

    let files: Vec<&str> = args.files.iter().map(|string| string.as_str()).collect();

    let request = FilesRequest {
        base: request_base!(progress),
        tmp_path: &ctx.tmp_path,
        subject: &subject,
        message: args.message.as_deref(),
        files,
        file_kind: FileDiffKind::Added,
        scope: args.scope.into(),
        file_package: &package,
        boot_plugin: &boot_plugin,
    };

    call!(symbols.files, request)
}
