// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::Args as ClapArgs;

use i18n_embed_fl::fl;

use upac_types::error::ErrorDomain;
use upac_types::package::PackageInfo;
use upac_types::request::mutated::{AttachRequest, FileTransfer};

use crate::locale::SUBJECT_LOADER;
use crate::types::abi::FileScope;
use crate::types::progress::ProgressState;
use crate::types::{CommandContext, boot_plugin, call, request_base};

#[derive(ClapArgs)]
pub struct Args {
    #[arg(num_args = 1.., required_unless_present = "source", conflicts_with_all = ["source", "target"])]
    pub files: Vec<String>,
    #[arg(long, requires = "target")]
    pub source: Option<String>,
    #[arg(long, requires = "source")]
    pub target: Option<String>,
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

impl Args {
    fn transfers(&self) -> Vec<FileTransfer<'_>> {
        if let (Some(source), Some(target)) = (&self.source, &self.target) {
            return vec![FileTransfer { source, target }];
        }

        self.files
            .iter()
            .map(|file| match file.split_once(':') {
                Some((source, target)) => FileTransfer { source, target },
                None => FileTransfer {
                    source: file,
                    target: file,
                },
            })
            .collect()
    }
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let symbols = ctx.lib.require_write()?;

    let mut progress = ProgressState::new(ErrorDomain::Attach);

    let boot_plugin = boot_plugin!(args.boot.clone())?;

    let subject = fl!(SUBJECT_LOADER, "subject-file-add");

    let request = AttachRequest {
        base: request_base!(progress),
        subject: &subject,
        message: args.message.as_deref(),
        files: args.transfers(),
        file_package: PackageInfo {
            name: args.package.clone(),
            arch: args.arch.clone(),
            arch_sub: args.arch_sub.clone(),
        },
        boot_plugin: &boot_plugin,
        scope: args.scope.into(),
    };

    call!(symbols.attach, request)
}
