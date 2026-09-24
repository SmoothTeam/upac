// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::{Path, PathBuf};

use super::{partition_node_path, split_partition_device};

use crate::commands::partition::error::PartitionError;

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
