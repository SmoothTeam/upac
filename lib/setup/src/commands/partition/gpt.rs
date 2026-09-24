// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::File;
use std::path::{Path, PathBuf};

use gptman::linux::{get_sector_size, reread_partition_table};
use gptman::{GPT, GPTPartitionEntry};

use uuid::{Uuid, uuid};

use upac_abi::PartitionKind;

use super::error::PartitionError;

use crate::layout::partition::FALLBACK_SECTOR_SIZE;

#[cfg(test)]
#[path = "../../../tests/inline/partition_gpt.rs"]
mod tests;

macro_rules! mib_to_sectors {
    ($size_mib:expr, $sector_size:expr) => {
        $size_mib * 1024 * 1024 / $sector_size
    };
}

const LABEL_MAX_UTF16_UNITS: usize = 36;

const ESP_PARTITION_TYPE_GUID: Uuid = uuid!("c12a7328-f81f-11d2-ba4b-00a0c93ec93b");
const LINUX_PARTITION_TYPE_GUID: Uuid = uuid!("0fc63daf-8483-4772-8e79-3d69d8477de4");
#[cfg(target_arch = "x86_64")]
const ROOT_PARTITION_TYPE_GUID: Uuid = uuid!("4f68bce3-e8cd-4db1-96e7-fbcaf984b709");
#[cfg(target_arch = "aarch64")]
const ROOT_PARTITION_TYPE_GUID: Uuid = uuid!("b921b045-1df0-41c3-af44-4c6f280d3fae");
#[cfg(target_arch = "riscv64")]
const ROOT_PARTITION_TYPE_GUID: Uuid = uuid!("72ec70a6-cf74-40e6-bd49-4bda08e8f224");

#[repr(transparent)]
pub(crate) struct GptTable(GPT);

impl GptTable {
    pub fn create(device: &mut File) -> Result<Self, PartitionError> {
        let sector_size = get_sector_size(device).unwrap_or(u64::from(FALLBACK_SECTOR_SIZE));

        Ok(GptTable(GPT::new_from(
            device,
            sector_size,
            Uuid::new_v4().to_bytes_le(),
        )?))
    }

    pub fn open(device: &mut File) -> Result<Self, PartitionError> {
        Ok(GptTable(GPT::find_from(device)?))
    }

    pub fn insert(&mut self, kind: PartitionKind, label: &str, size_mib: u64) -> Result<u32, PartitionError> {
        if label.encode_utf16().count() > LABEL_MAX_UTF16_UNITS {
            return Err(PartitionError::LabelTooLong);
        }

        let size_sectors = mib_to_sectors!(size_mib, self.0.sector_size);
        if size_sectors == 0 {
            return Err(PartitionError::InvalidSize);
        }

        let number = self
            .0
            .iter()
            .find(|(_, entry)| entry.is_unused())
            .map(|(number, _)| number)
            .ok_or(PartitionError::NoFreeSlot)?;

        let starting_lba = self
            .0
            .find_first_place(size_sectors)
            .ok_or(PartitionError::NoSpaceLeft)?;

        let partition_type = match kind {
            PartitionKind::Esp => ESP_PARTITION_TYPE_GUID,
            PartitionKind::Root => ROOT_PARTITION_TYPE_GUID,
            PartitionKind::Linux => LINUX_PARTITION_TYPE_GUID,
        };

        self.0[number] = GPTPartitionEntry {
            partition_type_guid: partition_type.to_bytes_le(),
            unique_partition_guid: Uuid::new_v4().to_bytes_le(),
            starting_lba,
            ending_lba: starting_lba + size_sectors - 1,
            attribute_bits: 0,
            partition_name: label.into(),
        };

        Ok(number)
    }

    pub fn write_protective_mbr_into(&self, device: &mut File) -> Result<(), PartitionError> {
        GPT::write_protective_mbr_into(device, self.0.sector_size)?;

        Ok(())
    }

    pub fn write_into(&mut self, device: &mut File) -> Result<(), PartitionError> {
        self.0.write_into(device)?;
        reread_partition_table(device)?;

        Ok(())
    }
}

pub(crate) fn partition_node_path(device_path: &Path, number: u32) -> PathBuf {
    let ends_in_digit = device_path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.chars().next_back())
        .is_some_and(|last| last.is_ascii_digit());

    let separator = if ends_in_digit { "p" } else { "" };

    PathBuf::from(format!("{}{separator}{number}", device_path.display()))
}

pub(crate) fn existing_esp_geometry(esp_device: &Path) -> Result<(u32, u64, u64, Uuid), PartitionError> {
    let (esp_partition, entry) = find_partition_entry(esp_device)?;

    Ok((
        esp_partition,
        entry.starting_lba,
        entry.ending_lba,
        Uuid::from_bytes_le(entry.unique_partition_guid),
    ))
}

pub(crate) fn is_esp_partition(partition_device: &Path) -> Result<bool, PartitionError> {
    let (_, entry) = find_partition_entry(partition_device)?;

    Ok(Uuid::from_bytes_le(entry.partition_type_guid) == ESP_PARTITION_TYPE_GUID)
}

fn find_partition_entry(partition_device: &Path) -> Result<(u32, GPTPartitionEntry), PartitionError> {
    let (disk_path, partition_number) = split_partition_device(partition_device)?;

    let mut device = File::open(disk_path)?;
    let gpt = GPT::find_from(&mut device)?;

    gpt.iter()
        .find(|&(number, entry)| number == partition_number && entry.is_used())
        .map(|(number, entry)| (number, entry.clone()))
        .ok_or(PartitionError::InvalidPartitionLayout)
}

fn split_partition_device(device: &Path) -> Result<(PathBuf, u32), PartitionError> {
    let name = device
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(PartitionError::InvalidPartitionLayout)?;

    let digits_start = name.len() - name.chars().rev().take_while(char::is_ascii_digit).count();
    if digits_start == name.len() {
        return Err(PartitionError::InvalidPartitionLayout);
    }

    let partition_number: u32 = name[digits_start..]
        .parse()
        .map_err(|_| PartitionError::InvalidPartitionLayout)?;

    let mut disk_name = &name[..digits_start];
    if let Some(prefix) = disk_name.strip_suffix('p')
        && prefix.chars().next_back().is_some_and(|last| last.is_ascii_digit())
    {
        disk_name = prefix;
    }

    Ok((device.with_file_name(disk_name), partition_number))
}
