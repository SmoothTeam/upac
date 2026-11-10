// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::{Args as ClapArgs, Subcommand};

use i18n_embed_fl::fl;

use upac_types::error::ErrorDomain;
use upac_types::request::partition::{PartitionKind, SetupPartitionAddRequest};
use upac_types::response::partition::SetupPartitionAddResponse;

use crate::libcore::Lib;
use crate::locale::LOADER;
use crate::types::output::Output;
use crate::types::progress::ProgressState;
use crate::types::{query, request_base};

pub mod create;
pub mod esp;
pub mod init;

#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: PartitionCommand,
}

#[derive(Subcommand)]
pub enum PartitionCommand {
    Init(init::Args),
    Esp(esp::Args),
    Create(create::Args),
}

pub fn run(args: Args, lib: &Lib, output: &Output) -> Result<()> {
    match args.command {
        PartitionCommand::Init(args) => init::run(args, lib),
        PartitionCommand::Esp(args) => esp::run(args, lib, output),
        PartitionCommand::Create(args) => create::run(args, lib, output),
    }
}

fn add(lib: &Lib, output: &Output, device: &str, label: &str, size_mib: u64, kind: PartitionKind) -> Result<()> {
    lib.require_root()?;

    let response = {
        let mut progress = ProgressState::new(ErrorDomain::PartitionAdd);

        query!(
            lib.partition_add,
            SetupPartitionAddRequest {
                base: request_base!(progress),
                device_path: device,
                label,
                size_mib,
                kind,
            } => SetupPartitionAddResponse
        )?
    };

    output.print(&[("label", &response.label), ("device", &response.device_path)], || {
        fl!(
            LOADER,
            "partition-created",
            label = response.label.as_str(),
            device = response.device_path.as_str()
        )
    });

    Ok(())
}
