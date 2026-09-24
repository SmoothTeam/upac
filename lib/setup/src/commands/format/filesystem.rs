// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::path::Path;
use std::process::Command;

use btrfs_mkfs::args::Profile;
use btrfs_mkfs::mkfs::{DeviceInfo, MkfsConfig, device_size, make_btrfs};
use btrfs_mkfs::write::ChecksumType;

use fatfs::{FatType, FormatVolumeOptions, format_volume};

use uuid::Uuid;

use upac_abi::FsKind;

use super::error::FormatError;

use crate::layout::mkfs::{EXT4_BIN, XFS_BIN};

macro_rules! fat_label {
    ($label:expr) => {{
        let mut bytes = [b' '; 11];
        for (slot, byte) in bytes.iter_mut().zip($label.as_bytes()) {
            *slot = byte.to_ascii_uppercase();
        }

        bytes
    }};
}

macro_rules! run_mkfs {
    ($binary:expr, $args:expr) => {{
        let status = Command::new($binary).args($args).status()?;

        if !status.success() {
            return Err(FormatError::MkfsFailed);
        }

        Ok(())
    }};
}

#[cfg(test)]
#[path = "../../../tests/inline/format.rs"]
mod tests;

pub(crate) struct FormatTarget<'target> {
    pub device_path: &'target Path,
    pub label: Option<&'target str>,
}

impl FormatTarget<'_> {
    pub fn format(&self, fs_kind: FsKind, btrfs_node_size: u32, btrfs_sector_size: u32) -> Result<(), FormatError> {
        match fs_kind {
            FsKind::Vfat => self.format_vfat(),
            FsKind::Ext4 => self.format_ext4(),
            FsKind::Btrfs => self.format_btrfs(btrfs_node_size, btrfs_sector_size),
            FsKind::Xfs => self.format_xfs(),
        }
    }

    fn format_vfat(&self) -> Result<(), FormatError> {
        let file = OpenOptions::new().read(true).write(true).open(self.device_path)?;

        let mut options = FormatVolumeOptions::new().fat_type(FatType::Fat32);
        if let Some(label) = self.label {
            options = options.volume_label(fat_label!(label));
        }

        format_volume(file, options)?;

        Ok(())
    }

    fn format_btrfs(&self, node_size: u32, sector_size: u32) -> Result<(), FormatError> {
        if !node_size.is_power_of_two() || !sector_size.is_power_of_two() {
            return Err(FormatError::InvalidFormatParams);
        }

        let total_bytes = device_size(self.device_path)?;
        let total_bytes = total_bytes / u64::from(sector_size) * u64::from(sector_size);

        let mut config = MkfsConfig {
            nodesize: node_size,
            sectorsize: sector_size,
            devices: vec![DeviceInfo {
                devid: 1,
                path: self.device_path.to_owned(),
                total_bytes,
                dev_uuid: Uuid::new_v4(),
            }],
            label: self.label.map(str::to_owned),
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

    fn format_ext4(&self) -> Result<(), FormatError> {
        let mut args = Vec::new();
        if let Some(label) = self.label {
            args.push(OsStr::new("-L"));
            args.push(OsStr::new(label));
        }
        args.push(self.device_path.as_os_str());

        run_mkfs!(EXT4_BIN, &args)
    }

    fn format_xfs(&self) -> Result<(), FormatError> {
        let mut args = vec![OsStr::new("-f")];
        if let Some(label) = self.label {
            args.push(OsStr::new("-L"));
            args.push(OsStr::new(label));
        }
        args.push(self.device_path.as_os_str());

        run_mkfs!(XFS_BIN, &args)
    }
}
