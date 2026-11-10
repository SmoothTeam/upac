// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use clap::Args as ClapArgs;

use upac_types::error::ErrorDomain;
use upac_types::request::format::{FsKind, SetupFormatRequest};

use crate::layout::disk_defaults::{BTRFS_NODE_SIZE, BTRFS_SECTOR_SIZE, ESP_LABEL};
use crate::libcore::Lib;
use crate::types::progress::ProgressState;
use crate::types::{call, request_base};

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long)]
    pub device: String,
    #[arg(long, default_value = ESP_LABEL)]
    pub label: String,
    #[arg(long)]
    pub force_wipe: bool,
}

pub fn run(args: Args, lib: &Lib) -> Result<()> {
    lib.require_root()?;

    let mut progress = ProgressState::new(ErrorDomain::Format);

    call!(
        lib.format_partition,
        SetupFormatRequest {
            base: request_base!(progress),
            device_path: &args.device,
            label: Some(args.label.as_str()),
            fs_kind: FsKind::Vfat,
            require_esp: true,
            force_wipe: args.force_wipe,
            btrfs_node_size: BTRFS_NODE_SIZE,
            btrfs_sector_size: BTRFS_SECTOR_SIZE,
        }
    )
}
