// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, bail};

use clap::{Args as ClapArgs, Subcommand, ValueEnum};

use i18n_embed_fl::fl;

use upac_abi::FsKind as FsKindAbi;
use upac_abi::error::ErrorDomain;

use upac_types::request::format::SetupFormatRequest;

use crate::layout::disk_defaults::{BTRFS_NODE_SIZE, BTRFS_SECTOR_SIZE, DEPLOY_FS, ESP_LABEL};
use crate::libcore::Lib;
use crate::locale::LOADER;
use crate::types::progress::ProgressState;
use crate::types::{FsKind, call, request_base};

#[cfg(test)]
#[path = "../../tests/inline/format.rs"]
mod tests;

#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: FormatCommand,
}

#[derive(Subcommand)]
pub enum FormatCommand {
    Esp(EspArgs),
    Create(CreateArgs),
}

#[derive(ClapArgs)]
pub struct EspArgs {
    #[arg(long)]
    pub device: Option<String>,
    #[arg(long, default_value = ESP_LABEL)]
    pub label: String,
    #[arg(long)]
    pub force_wipe: bool,
}

#[derive(ClapArgs)]
pub struct CreateArgs {
    #[arg(long)]
    pub device: Option<String>,
    #[arg(long, value_enum, default_value_t = FsKind::from_str(DEPLOY_FS, false).unwrap_or(FsKind(FsKindAbi::Btrfs)))]
    pub fs: FsKind,
    #[arg(long)]
    pub label: Option<String>,
    #[arg(long)]
    pub force_wipe: bool,
    #[arg(long, default_value_t = BTRFS_NODE_SIZE)]
    pub btrfs_node_size: u32,
    #[arg(long, default_value_t = BTRFS_SECTOR_SIZE)]
    pub btrfs_sector_size: u32,
}

pub fn run(args: Args, lib: &Lib) -> Result<()> {
    match args.command {
        FormatCommand::Esp(args) => esp(args, lib),
        FormatCommand::Create(args) => create(args, lib),
    }
}

fn esp(args: EspArgs, lib: &Lib) -> Result<()> {
    let Some(device) = args.device else {
        bail!(fl!(LOADER, "err-missing-device"));
    };

    lib.require_root()?;

    let mut progress = ProgressState::new(ErrorDomain::Format);

    call!(
        lib.format_partition,
        SetupFormatRequest {
            base: request_base!(progress),
            device_path: &device,
            label: Some(args.label.as_str()),
            fs_kind: FsKindAbi::Vfat,
            require_esp: true,
            force_wipe: args.force_wipe,
            btrfs_node_size: BTRFS_NODE_SIZE,
            btrfs_sector_size: BTRFS_SECTOR_SIZE,
        }
    )
}

fn create(args: CreateArgs, lib: &Lib) -> Result<()> {
    let Some(device) = args.device else {
        bail!(fl!(LOADER, "err-missing-device"));
    };

    lib.require_root()?;

    let mut progress = ProgressState::new(ErrorDomain::Format);

    call!(
        lib.format_partition,
        SetupFormatRequest {
            base: request_base!(progress),
            device_path: &device,
            label: args.label.as_deref(),
            fs_kind: args.fs.into(),
            require_esp: false,
            force_wipe: args.force_wipe,
            btrfs_node_size: args.btrfs_node_size,
            btrfs_sector_size: args.btrfs_sector_size,
        }
    )
}
