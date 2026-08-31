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

use crate::error::SetupError;
use crate::layout::btrfs::{NODE_SIZE, SECTOR_SIZE};
use crate::layout::mkfs::{EXT4_BIN, WIPEFS_BIN, XFS_BIN};

pub struct FormatTarget<'a> {
    pub device_path: &'a Path,
    pub label: Option<&'a str>,
}

impl FormatTarget<'_> {
    pub fn format(
        &self, fs_kind: FsKind, node_size: u32, sector_size: u32, force_wipe: bool,
    ) -> Result<(), SetupError> {
        if force_wipe {
            self.wipe_signature()?;
        }

        match fs_kind {
            FsKind::Ext4 => self.format_ext4(),
            FsKind::Btrfs => self.format_btrfs(node_size, sector_size),
            FsKind::Xfs => self.format_xfs(),
        }
    }

    pub fn wipe_signature(&self) -> Result<(), SetupError> {
        let status = Command::new(WIPEFS_BIN)
            .args(["-a", &self.device_path.to_string_lossy()])
            .status()?;

        if !status.success() {
            return Err(SetupError::WipeFailed);
        }

        Ok(())
    }

    pub fn format_esp(&self) -> Result<(), SetupError> {
        let file = OpenOptions::new().read(true).write(true).open(self.device_path)?;

        let mut options = FormatVolumeOptions::new().fat_type(FatType::Fat32);
        if let Some(label) = self.label {
            options = options.volume_label(fat_label(label));
        }

        format_volume(file, options)?;

        Ok(())
    }

    pub fn format_btrfs(&self, node_size: u32, sector_size: u32) -> Result<(), SetupError> {
        let node_size = if node_size == 0 { NODE_SIZE } else { node_size };
        let sector_size = if sector_size == 0 { SECTOR_SIZE } else { sector_size };

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

    pub fn format_ext4(&self) -> Result<(), SetupError> {
        let mut args = Vec::new();
        if let Some(label) = self.label {
            args.push(OsStr::new("-L"));
            args.push(OsStr::new(label));
        }
        args.push(self.device_path.as_os_str());

        run_mkfs(EXT4_BIN, &args)
    }

    pub fn format_xfs(&self) -> Result<(), SetupError> {
        let mut args = vec![OsStr::new("-f")];
        if let Some(label) = self.label {
            args.push(OsStr::new("-L"));
            args.push(OsStr::new(label));
        }
        args.push(self.device_path.as_os_str());

        run_mkfs(XFS_BIN, &args)
    }
}

fn fat_label(label: &str) -> [u8; 11] {
    let mut bytes = [b' '; 11];
    for (slot, byte) in bytes.iter_mut().zip(label.as_bytes()) {
        *slot = byte.to_ascii_uppercase();
    }

    bytes
}

fn run_mkfs(binary: &str, args: &[&OsStr]) -> Result<(), SetupError> {
    let status = Command::new(binary).args(args).status()?;

    if !status.success() {
        return Err(SetupError::MkfsFailed);
    }

    Ok(())
}
