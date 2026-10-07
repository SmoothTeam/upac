// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::{Args as ClapArgs, ValueEnum};

use upac_types::error::ErrorDomain;
use upac_types::request::format::{FsKind as FsKindAbi, SetupFormatRequest};

use crate::layout::disk_defaults::{BTRFS_NODE_SIZE, BTRFS_SECTOR_SIZE, DEPLOY_FS};
use crate::libcore::Lib;
use crate::types::progress::ProgressState;
use crate::types::{FsKind, call, request_base};

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long)]
    pub device: String,
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
    lib.require_root()?;

    let mut progress = ProgressState::new(ErrorDomain::Format);

    call!(
        lib.format_partition,
        SetupFormatRequest {
            base: request_base!(progress),
            device_path: &args.device,
            label: args.label.as_deref(),
            fs_kind: args.fs.into(),
            require_esp: false,
            force_wipe: args.force_wipe,
            btrfs_node_size: args.btrfs_node_size,
            btrfs_sector_size: args.btrfs_sector_size,
        }
    )
}
