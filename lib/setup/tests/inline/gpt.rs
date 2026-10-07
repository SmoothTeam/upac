// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use upac_types::error::ErrorKind;
use upac_types::request::partition::PartitionKind;

use super::{Disk, partition_node_path, split_partition_device};

fn blank_disk_image(scratch: &TempDir) -> PathBuf {
    let disk_path = scratch.path().join("disk");

    let disk = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&disk_path)
        .unwrap();
    disk.set_len(64 * 1024 * 1024).unwrap();

    disk_path
}

fn write_table(disk: &mut Disk) {
    let mut device = OpenOptions::new().read(true).write(true).open(&disk.path).unwrap();
    disk.table.write_into(&mut device).unwrap();
}

fn disk_image_with(scratch: &TempDir, kinds: &[PartitionKind]) -> PathBuf {
    let disk_path = blank_disk_image(scratch);

    let mut disk = Disk::create(&disk_path).unwrap();
    for kind in kinds {
        disk.add(*kind, "part", 4).unwrap();
    }
    write_table(&mut disk);

    disk_path
}

fn found_path(disk_path: &Path, kind: PartitionKind) -> Result<PathBuf, ErrorKind> {
    Disk::open(disk_path)?
        .find(kind)
        .map(|partition| partition.path().to_path_buf())
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
        Err(ErrorKind::NotAPartition)
    );
}

#[test]
fn find_returns_the_single_matching_partition() {
    let scratch = TempDir::new().unwrap();
    let disk_path = disk_image_with(&scratch, &[PartitionKind::Esp, PartitionKind::Root]);

    assert_eq!(
        found_path(&disk_path, PartitionKind::Esp),
        Ok(partition_node_path(&disk_path, 1))
    );
    assert_eq!(
        found_path(&disk_path, PartitionKind::Root),
        Ok(partition_node_path(&disk_path, 2))
    );
}

#[test]
fn find_reports_a_missing_kind() {
    let scratch = TempDir::new().unwrap();
    let disk_path = disk_image_with(&scratch, &[PartitionKind::Esp, PartitionKind::Linux]);

    assert_eq!(found_path(&disk_path, PartitionKind::Root), Err(ErrorKind::NotFound));
}

#[test]
fn find_refuses_to_guess_between_several_matches() {
    let scratch = TempDir::new().unwrap();
    let disk_path = disk_image_with(
        &scratch,
        &[PartitionKind::Esp, PartitionKind::Root, PartitionKind::Root],
    );

    assert_eq!(
        found_path(&disk_path, PartitionKind::Root),
        Err(ErrorKind::InvalidEntry)
    );
}

#[test]
fn add_returns_the_new_partition_with_its_node_path() {
    let scratch = TempDir::new().unwrap();
    let disk_path = blank_disk_image(&scratch);

    let mut disk = Disk::create(&disk_path).unwrap();
    let esp = disk.add(PartitionKind::Esp, "esp", 4).unwrap();
    let root = disk.add(PartitionKind::Root, "root", 4).unwrap();

    assert_eq!(esp.number(), 1);
    assert_eq!(esp.path(), partition_node_path(&disk_path, 1));
    assert!(esp.is_esp());
    assert_eq!(root.number(), 2);
    assert!(!root.is_esp());
    assert!(root.first_lba() > esp.last_lba());
}

#[test]
fn add_rejects_a_label_longer_than_gpt_allows() {
    let scratch = TempDir::new().unwrap();
    let mut disk = Disk::create(&blank_disk_image(&scratch)).unwrap();

    assert_eq!(
        disk.add(PartitionKind::Linux, &"x".repeat(37), 4).err(),
        Some(ErrorKind::InvalidEntry)
    );
}

#[test]
fn add_rejects_an_empty_partition() {
    let scratch = TempDir::new().unwrap();
    let mut disk = Disk::create(&blank_disk_image(&scratch)).unwrap();

    assert_eq!(
        disk.add(PartitionKind::Linux, "empty", 0).err(),
        Some(ErrorKind::InvalidEntry)
    );
}

#[test]
fn a_partition_keeps_what_was_written_for_it() {
    let scratch = TempDir::new().unwrap();
    let disk_path = blank_disk_image(&scratch);

    let mut disk = Disk::create(&disk_path).unwrap();
    let added = disk.add(PartitionKind::Esp, "esp", 4).unwrap();
    write_table(&mut disk);

    assert_eq!(Disk::open(&disk_path).unwrap().find(PartitionKind::Esp), Ok(added));
}
