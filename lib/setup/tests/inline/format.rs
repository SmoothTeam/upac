// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, OpenOptions};
use std::io::Read;

use tempfile::TempDir;

use super::FormatTarget;

#[test]
fn format_esp_writes_a_valid_fat32_boot_sector() {
    let scratch = TempDir::new().unwrap();
    let device_path = scratch.path().join("esp.img");

    {
        let file = File::create(&device_path).unwrap();
        file.set_len(64 * 1024 * 1024).unwrap();
    }

    let target = FormatTarget {
        device_path: &device_path,
        label: Some("ESP"),
    };
    target.format_esp().unwrap();

    let mut file = OpenOptions::new().read(true).open(&device_path).unwrap();
    let mut boot_sector = [0u8; 512];
    file.read_exact(&mut boot_sector).unwrap();

    assert_eq!(&boot_sector[510..512], &[0x55, 0xAA]);
}

#[test]
fn fat_label_pads_short_labels_with_trailing_spaces() {
    let label = fat_label!("esp");

    assert_eq!(&label[..3], b"ESP");
    assert_eq!(&label[3..], [b' '; 8]);
}

#[test]
fn fat_label_uppercases_input() {
    let label = fat_label!("boot");

    assert_eq!(&label[..4], b"BOOT");
    assert_eq!(&label[4..], [b' '; 7]);
}

#[test]
fn fat_label_truncates_labels_longer_than_eleven_bytes() {
    let label = fat_label!("a-very-long-label");

    assert_eq!(&label, b"A-VERY-LONG");
}
