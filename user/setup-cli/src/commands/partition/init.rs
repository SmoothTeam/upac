// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::Args as ClapArgs;

use upac_types::error::ErrorDomain;
use upac_types::request::partition::SetupPartitionTableRequest;

use crate::libcore::Lib;
use crate::types::progress::ProgressState;
use crate::types::{call, request_base};

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long)]
    pub device: String,
    #[arg(long)]
    pub force_wipe: bool,
}

pub fn run(args: Args, lib: &Lib) -> Result<()> {
    lib.require_root()?;

    let mut progress = ProgressState::new(ErrorDomain::PartitionTable);

    call!(
        lib.partition_table,
        SetupPartitionTableRequest {
            base: request_base!(progress),
            device_path: &args.device,
            force_wipe: args.force_wipe,
        }
    )
}
