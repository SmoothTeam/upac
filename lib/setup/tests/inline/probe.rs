// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::File;
use std::os::unix::fs::FileExt;
use std::path::PathBuf;

use tempfile::TempDir;

use upac_types::request::format::FsKind;

use super::{
    BTRFS_MAGIC, BTRFS_MAGIC_OFFSET, EXT4_MAGIC, EXT4_MAGIC_OFFSET, XFS_MAGIC, XFS_MAGIC_OFFSET, detect_filesystem,
};

fn image_with(scratch: &TempDir, length: u64, signature: Option<(u64, &[u8])>) -> PathBuf {
    let path = scratch.path().join("partition.img");

    let file = File::create(&path).unwrap();
    file.set_len(length).unwrap();
    if let Some((offset, magic)) = signature {
        file.write_all_at(magic, offset).unwrap();
    }

    path
}

#[test]
fn detects_each_supported_filesystem_by_its_signature() {
    for (fs_kind, offset, magic) in [
        (FsKind::Ext4, EXT4_MAGIC_OFFSET, EXT4_MAGIC),
        (FsKind::Btrfs, BTRFS_MAGIC_OFFSET, BTRFS_MAGIC),
        (FsKind::Xfs, XFS_MAGIC_OFFSET, XFS_MAGIC),
    ] {
        let scratch = TempDir::new().unwrap();
        let path = image_with(&scratch, 1024 * 1024, Some((offset, magic)));

        assert_eq!(detect_filesystem(&path).unwrap(), Some(fs_kind));
    }
}

#[test]
fn reports_nothing_for_an_unformatted_device() {
    let scratch = TempDir::new().unwrap();
    let path = image_with(&scratch, 1024 * 1024, None);

    assert_eq!(detect_filesystem(&path).unwrap(), None);
}

#[test]
fn a_device_too_short_for_a_signature_is_not_an_error() {
    let scratch = TempDir::new().unwrap();
    let path = image_with(&scratch, 2048, None);

    assert_eq!(detect_filesystem(&path).unwrap(), None);
}
