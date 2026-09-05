// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use clap::ValueEnum;
use clap::builder::PossibleValue;

use upac_abi::FsKind as FsKindAbi;
use upac_types::{PartitionMount, PartitionSpec};

#[cfg(test)]
#[path = "../tests/inline/types.rs"]
mod tests;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct FsKind(pub FsKindAbi);

impl From<FsKind> for FsKindAbi {
    fn from(value: FsKind) -> Self {
        value.0
    }
}

impl ValueEnum for FsKind {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            FsKind(FsKindAbi::Ext4),
            FsKind(FsKindAbi::Btrfs),
            FsKind(FsKindAbi::Xfs),
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self.0 {
            FsKindAbi::Ext4 => PossibleValue::new("ext4"),
            FsKindAbi::Btrfs => PossibleValue::new("btrfs"),
            FsKindAbi::Xfs => PossibleValue::new("xfs"),
        })
    }
}

pub fn parse_extra_mount(raw: &str) -> Result<PartitionMount, String> {
    let mut parts = raw.splitn(3, ':');

    let mount_path = parts.next().filter(|part| !part.is_empty());
    let device_path = parts.next().filter(|part| !part.is_empty());
    let fs_kind = parts.next().filter(|part| !part.is_empty());

    match (mount_path, device_path, fs_kind) {
        (Some(mount_path), Some(device_path), Some(fs_kind)) => Ok(PartitionMount {
            mount_path: mount_path.to_owned(),
            device_path: device_path.to_owned(),
            fs_kind: FsKind::from_str(fs_kind, false).map(FsKindAbi::from)?,
        }),
        _ => Err(format!("expected <mount_path>:<device_path>:<fs_kind>, got \"{raw}\"")),
    }
}

pub fn parse_size_mib(raw: &str) -> Result<u64, String> {
    let trimmed = raw.trim();

    let Some(split_at) = trimmed.find(|character: char| !character.is_ascii_digit()) else {
        return trimmed.parse().map_err(|_| format!("invalid size: \"{raw}\""));
    };

    let (number, unit) = trimmed.split_at(split_at);
    let number: u64 = number.parse().map_err(|_| format!("invalid size: \"{raw}\""))?;

    let bytes_per_unit: u64 = match unit.to_ascii_uppercase().as_str() {
        "K" | "KIB" => 1024,
        "M" | "MIB" => 1024 * 1024,
        "G" | "GIB" => 1024 * 1024 * 1024,
        "T" | "TIB" => 1024 * 1024 * 1024 * 1024,
        "KB" => 1_000,
        "MB" => 1_000_000,
        "GB" => 1_000_000_000,
        "TB" => 1_000_000_000_000,
        _ => return Err(format!("unknown size unit: \"{unit}\"")),
    };

    Ok(number * bytes_per_unit / (1024 * 1024))
}

pub fn parse_extra_partition(raw: &str) -> Result<PartitionSpec, String> {
    let mut parts = raw.splitn(3, ':');

    let mount_path = parts.next().filter(|part| !part.is_empty());
    let size_mib = parts.next().filter(|part| !part.is_empty());
    let fs_kind = parts.next().filter(|part| !part.is_empty());

    match (mount_path, size_mib, fs_kind) {
        (Some(mount_path), Some(size_mib), Some(fs_kind)) => Ok(PartitionSpec {
            mount_path: mount_path.to_owned(),
            size_mib: size_mib
                .parse()
                .map_err(|_| format!("invalid size_mib: \"{size_mib}\""))?,
            fs_kind: FsKind::from_str(fs_kind, false).map(FsKindAbi::from)?,
        }),
        _ => Err(format!("expected <mount_path>:<size_mib>:<fs_kind>, got \"{raw}\"")),
    }
}
