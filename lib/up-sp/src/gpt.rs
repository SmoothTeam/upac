// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, OpenOptions, canonicalize, read_to_string};
use std::path::{Path, PathBuf};

use gptman::linux::{get_sector_size, reread_partition_table};
use gptman::{GPT, GPTPartitionEntry};

use uuid::{Uuid, uuid};

use upac_types::error::ErrorKind;
use upac_types::request::partition::PartitionKind;

use crate::layout::partition::FALLBACK_SECTOR_SIZE;

#[cfg(test)]
#[path = "../tests/inline/gpt.rs"]
mod tests;

macro_rules! mib_to_sectors {
    ($size_mib:expr, $sector_size:expr) => {
        $size_mib * 1024 * 1024 / $sector_size
    };
}

const LABEL_MAX_UTF16_UNITS: usize = 36;
const SYSFS_BLOCK_DIR: &str = "/sys/class/block";

const ESP_PARTITION_TYPE_GUID: Uuid = uuid!("c12a7328-f81f-11d2-ba4b-00a0c93ec93b");
const LINUX_PARTITION_TYPE_GUID: Uuid = uuid!("0fc63daf-8483-4772-8e79-3d69d8477de4");
#[cfg(target_arch = "x86_64")]
const ROOT_PARTITION_TYPE_GUID: Uuid = uuid!("4f68bce3-e8cd-4db1-96e7-fbcaf984b709");
#[cfg(target_arch = "aarch64")]
const ROOT_PARTITION_TYPE_GUID: Uuid = uuid!("b921b045-1df0-41c3-af44-4c6f280d3fae");
#[cfg(target_arch = "riscv64")]
const ROOT_PARTITION_TYPE_GUID: Uuid = uuid!("72ec70a6-cf74-40e6-bd49-4bda08e8f224");

pub(crate) struct Disk {
    path: PathBuf,
    table: GPT,
    fresh: bool,
}

impl Disk {
    pub fn create(path: &Path) -> Result<Self, ErrorKind> {
        let mut device = File::open(path)?;
        let sector_size = get_sector_size(&mut device).unwrap_or(u64::from(FALLBACK_SECTOR_SIZE));

        Ok(Disk {
            path: path.to_path_buf(),
            table: GPT::new_from(&mut device, sector_size, Uuid::new_v4().to_bytes_le())?,
            fresh: true,
        })
    }

    pub fn open(path: &Path) -> Result<Self, ErrorKind> {
        let mut device = File::open(path)?;

        Ok(Disk {
            path: path.to_path_buf(),
            table: GPT::find_from(&mut device)?,
            fresh: false,
        })
    }

    pub fn add(&mut self, kind: PartitionKind, label: &str, size_mib: u64) -> Result<Partition, ErrorKind> {
        if label.encode_utf16().count() > LABEL_MAX_UTF16_UNITS {
            return Err(ErrorKind::InvalidEntry);
        }

        let size_sectors = mib_to_sectors!(size_mib, self.table.sector_size);
        if size_sectors == 0 {
            return Err(ErrorKind::InvalidEntry);
        }

        let number = self
            .table
            .iter()
            .find(|(_, entry)| entry.is_unused())
            .map(|(number, _)| number)
            .ok_or(ErrorKind::NoSpaceLeft)?;

        let starting_lba = self
            .table
            .find_first_place(size_sectors)
            .ok_or(ErrorKind::NoSpaceLeft)?;

        self.table[number] = GPTPartitionEntry {
            partition_type_guid: partition_type_guid(kind).to_bytes_le(),
            unique_partition_guid: Uuid::new_v4().to_bytes_le(),
            starting_lba,
            ending_lba: starting_lba + size_sectors - 1,
            attribute_bits: 0,
            partition_name: label.into(),
        };

        self.partition(number)
    }

    pub fn find(&self, kind: PartitionKind) -> Result<Partition, ErrorKind> {
        let type_guid = partition_type_guid(kind).to_bytes_le();

        let mut numbers = self
            .table
            .iter()
            .filter(|(_, entry)| entry.is_used() && entry.partition_type_guid == type_guid)
            .map(|(number, _)| number);

        let number = numbers.next().ok_or(ErrorKind::NotFound)?;
        if numbers.next().is_some() {
            return Err(ErrorKind::InvalidEntry);
        }

        self.partition(number)
    }

    pub fn commit(mut self) -> Result<(), ErrorKind> {
        let mut device = OpenOptions::new().read(true).write(true).open(&self.path)?;

        if self.fresh {
            GPT::write_protective_mbr_into(&mut device, self.table.sector_size)?;
        }

        self.table.write_into(&mut device)?;
        reread_partition_table(&mut device)?;

        Ok(())
    }

    fn partition(&self, number: u32) -> Result<Partition, ErrorKind> {
        let entry = self
            .table
            .iter()
            .find(|&(candidate, entry)| candidate == number && entry.is_used())
            .map(|(_, entry)| entry.clone())
            .ok_or(ErrorKind::InvalidEntry)?;

        Ok(Partition {
            path: partition_node_path(&self.path, number),
            number,
            entry,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Partition {
    path: PathBuf,
    number: u32,
    entry: GPTPartitionEntry,
}

impl Partition {
    pub fn open(device: &Path) -> Result<Self, ErrorKind> {
        let (disk_path, number) = split_partition_device(device)?;

        Disk::open(&disk_path)?.partition(number)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn is_esp(&self) -> bool {
        Uuid::from_bytes_le(self.entry.partition_type_guid) == ESP_PARTITION_TYPE_GUID
    }

    pub fn number(&self) -> u32 {
        self.number
    }

    pub fn first_lba(&self) -> u64 {
        self.entry.starting_lba
    }

    pub fn last_lba(&self) -> u64 {
        self.entry.ending_lba
    }

    pub fn unique_guid(&self) -> Uuid {
        Uuid::from_bytes_le(self.entry.unique_partition_guid)
    }
}

fn partition_node_path(device_path: &Path, number: u32) -> PathBuf {
    let ends_in_digit = device_path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.chars().next_back())
        .is_some_and(|last| last.is_ascii_digit());

    let separator = if ends_in_digit { "p" } else { "" };

    PathBuf::from(format!("{}{separator}{number}", device_path.display()))
}

fn partition_type_guid(kind: PartitionKind) -> Uuid {
    match kind {
        PartitionKind::Esp => ESP_PARTITION_TYPE_GUID,
        PartitionKind::Root => ROOT_PARTITION_TYPE_GUID,
        PartitionKind::Linux => LINUX_PARTITION_TYPE_GUID,
    }
}

fn split_partition_device(device: &Path) -> Result<(PathBuf, u32), ErrorKind> {
    let device = canonicalize(device).unwrap_or_else(|_| device.to_path_buf());
    let name = device
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(ErrorKind::NotAPartition)?;

    let sysfs_entry = Path::new(SYSFS_BLOCK_DIR).join(name);
    if sysfs_entry.exists() {
        return split_partition_device_by_sysfs(&device, &sysfs_entry);
    }

    split_partition_device_by_name(&device, name)
}

fn split_partition_device_by_sysfs(device: &Path, sysfs_entry: &Path) -> Result<(PathBuf, u32), ErrorKind> {
    let partition_number: u32 = read_to_string(sysfs_entry.join("partition"))
        .map_err(|_| ErrorKind::NotAPartition)?
        .trim()
        .parse()
        .map_err(|_| ErrorKind::NotAPartition)?;

    let resolved_entry = canonicalize(sysfs_entry)?;
    let disk_name = resolved_entry
        .parent()
        .and_then(|parent| parent.file_name())
        .ok_or(ErrorKind::NotAPartition)?;

    Ok((device.with_file_name(disk_name), partition_number))
}

fn split_partition_device_by_name(device: &Path, name: &str) -> Result<(PathBuf, u32), ErrorKind> {
    let digits_start = name.len() - name.chars().rev().take_while(char::is_ascii_digit).count();
    if digits_start == name.len() {
        return Err(ErrorKind::NotAPartition);
    }

    let partition_number: u32 = name[digits_start..].parse().map_err(|_| ErrorKind::NotAPartition)?;

    let mut disk_name = &name[..digits_start];
    if let Some(prefix) = disk_name.strip_suffix('p')
        && prefix.chars().next_back().is_some_and(|last| last.is_ascii_digit())
    {
        disk_name = prefix;
    }

    Ok((device.with_file_name(disk_name), partition_number))
}
