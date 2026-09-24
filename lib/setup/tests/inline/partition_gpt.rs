// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::File;
use std::path::{Path, PathBuf};

use gptman::GPT;

use tempfile::TempDir;

use uuid::Uuid;

use upac_abi::PartitionKind;

use super::{GptTable, find_partition_by_kind, partition_node_path, split_partition_device};

use crate::commands::partition::error::PartitionError;

fn disk_image_with(scratch: &TempDir, kinds: &[PartitionKind]) -> PathBuf {
    let disk_path = scratch.path().join("disk");

    let mut disk = File::create(&disk_path).unwrap();
    disk.set_len(64 * 1024 * 1024).unwrap();

    let mut table = GptTable(GPT::new_from(&mut disk, 512, Uuid::new_v4().to_bytes_le()).unwrap());
    for kind in kinds {
        table.insert(*kind, "part", 4).unwrap();
    }
    table.0.write_into(&mut disk).unwrap();

    disk_path
}

#[test]
fn partition_node_path_appends_the_number_for_a_plain_device_name() {
    assert_eq!(
        partition_node_path(Path::new("/dev/sda"), 1),
        PathBuf::from("/dev/sda1")
    );
    assert_eq!(
        partition_node_path(Path::new("/dev/sda"), 12),
        PathBuf::from("/dev/sda12")
    );
}

#[test]
fn partition_node_path_inserts_a_p_separator_when_the_device_name_ends_in_a_digit() {
    assert_eq!(
        partition_node_path(Path::new("/dev/nvme0n1"), 2),
        PathBuf::from("/dev/nvme0n1p2")
    );
    assert_eq!(
        partition_node_path(Path::new("/dev/mmcblk0"), 1),
        PathBuf::from("/dev/mmcblk0p1")
    );
}

#[test]
fn split_partition_device_is_the_inverse_of_partition_node_path() {
    for (disk, number) in [("/dev/sda", 3), ("/dev/nvme0n1", 2), ("/dev/mmcblk0", 1)] {
        let node = partition_node_path(Path::new(disk), number);

        assert_eq!(split_partition_device(&node), Ok((PathBuf::from(disk), number)));
    }
}

#[test]
fn split_partition_device_rejects_a_whole_disk_name_without_a_partition_number() {
    assert_eq!(
        split_partition_device(Path::new("/dev/sda")),
        Err(PartitionError::InvalidPartitionLayout)
    );
}

#[test]
fn find_partition_by_kind_returns_the_single_matching_node() {
    let scratch = TempDir::new().unwrap();
    let disk_path = disk_image_with(&scratch, &[PartitionKind::Esp, PartitionKind::Root]);

    assert_eq!(
        find_partition_by_kind(&disk_path, PartitionKind::Esp),
        Ok(partition_node_path(&disk_path, 1))
    );
    assert_eq!(
        find_partition_by_kind(&disk_path, PartitionKind::Root),
        Ok(partition_node_path(&disk_path, 2))
    );
}

#[test]
fn find_partition_by_kind_reports_a_missing_kind() {
    let scratch = TempDir::new().unwrap();
    let disk_path = disk_image_with(&scratch, &[PartitionKind::Esp, PartitionKind::Linux]);

    assert_eq!(
        find_partition_by_kind(&disk_path, PartitionKind::Root),
        Err(PartitionError::PartitionKindNotFound)
    );
}

#[test]
fn find_partition_by_kind_refuses_to_guess_between_several_matches() {
    let scratch = TempDir::new().unwrap();
    let disk_path = disk_image_with(
        &scratch,
        &[PartitionKind::Esp, PartitionKind::Root, PartitionKind::Root],
    );

    assert_eq!(
        find_partition_by_kind(&disk_path, PartitionKind::Root),
        Err(PartitionError::PartitionKindAmbiguous)
    );
}
