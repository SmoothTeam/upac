// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::path::Path;

use btrfs_mkfs::args::Profile;
use btrfs_mkfs::mkfs::{DeviceInfo, MkfsConfig, device_size, make_btrfs};
use btrfs_mkfs::write::ChecksumType;

use uuid::Uuid;

use crate::error::SetupError;
use crate::layout;

pub fn format_btrfs(
    device_path: &Path, label: Option<&str>, node_size: u32, sector_size: u32,
) -> Result<(), SetupError> {
    let node_size = if node_size == 0 {
        layout::btrfs::NODE_SIZE
    } else {
        node_size
    };
    let sector_size = if sector_size == 0 {
        layout::btrfs::SECTOR_SIZE
    } else {
        sector_size
    };

    let total_bytes = device_size(device_path)?;
    let total_bytes = total_bytes / u64::from(sector_size) * u64::from(sector_size);

    let mut config = MkfsConfig {
        nodesize: node_size,
        sectorsize: sector_size,
        devices: vec![DeviceInfo {
            devid: 1,
            path: device_path.to_owned(),
            total_bytes,
            dev_uuid: Uuid::new_v4(),
        }],
        label: label.map(str::to_owned),
        fs_uuid: Uuid::new_v4(),
        chunk_tree_uuid: Uuid::new_v4(),
        incompat_flags: MkfsConfig::default_incompat_flags(),
        compat_ro_flags: MkfsConfig::default_compat_ro_flags(),
        data_profile: Profile::Single,
        metadata_profile: Profile::Dup,
        csum_type: ChecksumType::Crc32,
        creation_time: None,
        quota: false,
        squota: false,
    };
    config.apply_profile_flags();

    make_btrfs(&config)?;

    Ok(())
}
