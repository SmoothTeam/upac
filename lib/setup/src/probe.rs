// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::File;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::os::unix::fs::FileExt;
use std::path::Path;

use upac_types::request::format::FsKind;

#[cfg(test)]
#[path = "../tests/inline/probe.rs"]
mod tests;

const EXT4_MAGIC_OFFSET: u64 = 1024 + 56;
const EXT4_MAGIC: &[u8] = &[0x53, 0xEF];

const BTRFS_MAGIC_OFFSET: u64 = 64 * 1024 + 64;
const BTRFS_MAGIC: &[u8] = b"_BHRfS_M";

const XFS_MAGIC_OFFSET: u64 = 0;
const XFS_MAGIC: &[u8] = b"XFSB";

const SIGNATURES: &[(FsKind, u64, &[u8])] = &[
    (FsKind::Xfs, XFS_MAGIC_OFFSET, XFS_MAGIC),
    (FsKind::Btrfs, BTRFS_MAGIC_OFFSET, BTRFS_MAGIC),
    (FsKind::Ext4, EXT4_MAGIC_OFFSET, EXT4_MAGIC),
];

pub(crate) fn detect_filesystem(device_path: &Path) -> Result<Option<FsKind>, IoError> {
    let device = File::open(device_path)?;

    for (fs_kind, offset, magic) in SIGNATURES {
        if has_signature(&device, *offset, magic)? {
            return Ok(Some(*fs_kind));
        }
    }

    Ok(None)
}

fn has_signature(device: &File, offset: u64, magic: &[u8]) -> Result<bool, IoError> {
    let mut found = vec![0u8; magic.len()];

    match device.read_exact_at(&mut found, offset) {
        Ok(()) => Ok(found == magic),
        Err(error) if error.kind() == IoErrorKind::UnexpectedEof => Ok(false),
        Err(error) => Err(error),
    }
}
