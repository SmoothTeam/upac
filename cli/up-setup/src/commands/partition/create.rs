// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::Args as ClapArgs;

use upac_types::request::partition::PartitionKind;

use super::add;

use crate::libcore::Lib;
use crate::types::output::Output;
use crate::types::{PartitionKindClapArg, parse_size_mib};

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long)]
    pub device: String,
    #[arg(long = "size", value_parser = parse_size_mib)]
    pub size_mib: u64,
    #[arg(long)]
    pub label: String,
    #[arg(long = "type", value_enum, default_value_t = PartitionKindClapArg(PartitionKind::Linux))]
    pub kind: PartitionKindClapArg,
}

pub fn run(args: Args, lib: &Lib, output: &Output) -> Result<()> {
    add(lib, output, &args.device, &args.label, args.size_mib, args.kind.into())
}
