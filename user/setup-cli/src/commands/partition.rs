// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, bail};

use clap::{Args as ClapArgs, Subcommand};

use i18n_embed_fl::fl;

use upac_abi::PartitionKind;
use upac_abi::error::ErrorDomain;

use upac_types::request::partition::{SetupPartitionAddRequest, SetupPartitionTableRequest};
use upac_types::response::partition::SetupPartitionAddResponse;

use crate::layout::disk_defaults::{ESP_LABEL, ESP_SIZE_MIB};
use crate::libcore::Lib;
use crate::locale::LOADER;
use crate::types::progress::ProgressState;
use crate::types::{PartitionKindClapArg, call, parse_size_mib, query, request_base};

#[cfg(test)]
#[path = "../../tests/inline/partition.rs"]
mod tests;

#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: PartitionCommand,
}

#[derive(Subcommand)]
pub enum PartitionCommand {
    Init(InitArgs),
    Esp(EspArgs),
    Create(CreateArgs),
}

#[derive(ClapArgs)]
pub struct InitArgs {
    #[arg(long)]
    pub device: Option<String>,
    #[arg(long)]
    pub force_wipe: bool,
}

#[derive(ClapArgs)]
pub struct EspArgs {
    #[arg(long)]
    pub device: Option<String>,
    #[arg(long = "size", value_parser = parse_size_mib, default_value_t = u64::from(ESP_SIZE_MIB))]
    pub size_mib: u64,
    #[arg(long, default_value = ESP_LABEL)]
    pub label: String,
}

#[derive(ClapArgs)]
pub struct CreateArgs {
    #[arg(long)]
    pub device: Option<String>,
    #[arg(long = "size", value_parser = parse_size_mib)]
    pub size_mib: Option<u64>,
    #[arg(long)]
    pub label: Option<String>,
    #[arg(long = "type", value_enum, default_value_t = PartitionKindClapArg(PartitionKind::Linux))]
    pub kind: PartitionKindClapArg,
}

pub fn run(args: Args, lib: &Lib) -> Result<()> {
    match args.command {
        PartitionCommand::Init(args) => init(args, lib),
        PartitionCommand::Esp(args) => esp(args, lib),
        PartitionCommand::Create(args) => create(args, lib),
    }
}

fn init(args: InitArgs, lib: &Lib) -> Result<()> {
    let Some(device) = args.device else {
        bail!(fl!(LOADER, "err-missing-device"));
    };

    lib.require_root()?;

    let mut progress = ProgressState::new(ErrorDomain::PartitionTable);

    call!(
        lib.partition_table,
        SetupPartitionTableRequest {
            base: request_base!(progress),
            device_path: &device,
            force_wipe: args.force_wipe,
        }
    )
}

fn esp(args: EspArgs, lib: &Lib) -> Result<()> {
    let Some(device) = args.device else {
        bail!(fl!(LOADER, "err-missing-device"));
    };

    add(lib, &device, &args.label, args.size_mib, PartitionKind::Esp)
}

fn create(args: CreateArgs, lib: &Lib) -> Result<()> {
    let Some(device) = args.device else {
        bail!(fl!(LOADER, "err-missing-device"));
    };
    let Some(size_mib) = args.size_mib else {
        bail!(fl!(LOADER, "err-missing-size"));
    };
    let Some(label) = args.label else {
        bail!(fl!(LOADER, "err-missing-label"));
    };

    add(lib, &device, &label, size_mib, args.kind.into())
}

fn add(lib: &Lib, device: &str, label: &str, size_mib: u64, kind: PartitionKind) -> Result<()> {
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

    println!(
        "{}",
        fl!(
            LOADER,
            "partition-created",
            label = response.label,
            device = response.device_path
        )
    );

    Ok(())
}
