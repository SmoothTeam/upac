// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use clap::ValueEnum;

use upac_abi::FsKind as FsKindAbi;

use super::{FsKind, parse_extra_mount, parse_extra_partition, parse_size_mib};

#[test]
fn parse_extra_mount_accepts_a_well_formed_triple() {
    let mount = parse_extra_mount("/mnt/boot:/dev/sda1:ext4").unwrap();

    assert_eq!(mount.mount_path, "/mnt/boot");
    assert_eq!(mount.device_path, "/dev/sda1");
    assert_eq!(mount.fs_kind, FsKindAbi::Ext4);
}

#[test]
fn parse_extra_mount_rejects_a_missing_field() {
    assert!(parse_extra_mount("/mnt/boot:/dev/sda1").is_err());
}

#[test]
fn parse_extra_mount_rejects_an_unknown_fs_kind() {
    assert!(parse_extra_mount("/mnt/boot:/dev/sda1:zfs").is_err());
}

#[test]
fn parse_extra_partition_accepts_a_well_formed_triple() {
    let partition = parse_extra_partition("/mnt/data:2048:btrfs").unwrap();

    assert_eq!(partition.mount_path, "/mnt/data");
    assert_eq!(partition.size_mib, 2048);
    assert_eq!(partition.fs_kind, FsKindAbi::Btrfs);
}

#[test]
fn parse_extra_partition_rejects_a_non_numeric_size() {
    assert!(parse_extra_partition("/mnt/data:big:btrfs").is_err());
}

#[test]
fn parse_extra_partition_rejects_a_missing_field() {
    assert!(parse_extra_partition("/mnt/data:2048").is_err());
}

#[test]
fn parse_size_mib_accepts_a_plain_number_as_mib() {
    assert_eq!(parse_size_mib("1024").unwrap(), 1024);
}

#[test]
fn parse_size_mib_converts_binary_units() {
    assert_eq!(parse_size_mib("1G").unwrap(), 1024);
    assert_eq!(parse_size_mib("512K").unwrap(), 0);
}

#[test]
fn parse_size_mib_converts_decimal_units() {
    assert_eq!(parse_size_mib("1GB").unwrap(), 953);
}

#[test]
fn parse_size_mib_rejects_an_unknown_unit() {
    assert!(parse_size_mib("10X").is_err());
}

#[test]
fn parse_size_mib_rejects_a_non_numeric_value() {
    assert!(parse_size_mib("abc").is_err());
}

#[test]
fn fs_kind_has_exactly_the_three_supported_variants() {
    assert_eq!(FsKind::value_variants().len(), 3);
}

#[test]
fn fs_kind_from_str_matches_lowercase_names() {
    assert!(FsKind::from_str("ext4", false).is_ok());
    assert!(FsKind::from_str("btrfs", false).is_ok());
    assert!(FsKind::from_str("xfs", false).is_ok());
    assert!(FsKind::from_str("EXT4", false).is_err());
}
