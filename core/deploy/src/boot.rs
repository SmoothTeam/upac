// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsStr;
use std::path::PathBuf;

use composefs::generic_tree::Stat;
use composefs::tree::{Directory, FileSystem, Inode};

use rsmount::tables::MountInfo;

use upac_composefs::ObjectID;

use super::error::EspError;
use super::layout::boot::{ESP_MOUNT_FALLBACK, ESP_MOUNT_PRIMARY};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WrittenBootEntry {
    Bls(String),
    Uki(String),
}

impl WrittenBootEntry {
    pub fn entry_name(&self) -> &str {
        match self {
            WrittenBootEntry::Bls(name) | WrittenBootEntry::Uki(name) => name,
        }
    }

    pub fn into_entry_name(self) -> String {
        match self {
            WrittenBootEntry::Bls(name) | WrittenBootEntry::Uki(name) => name,
        }
    }
}

pub fn find_esp_mount() -> Result<PathBuf, EspError> {
    let mut mount_table = MountInfo::new()?;
    mount_table.import_mountinfo()?;

    for candidate_for_mount in [ESP_MOUNT_PRIMARY, ESP_MOUNT_FALLBACK] {
        if mount_table.find_target(candidate_for_mount).is_some() {
            return Ok(PathBuf::from(candidate_for_mount));
        }
    }

    Err(EspError::NotFound)
}

pub(crate) fn wrap_under_usr(tree: &FileSystem<ObjectID>) -> FileSystem<ObjectID> {
    let mut root = Directory::new(Stat::uninitialized());
    root.insert(OsStr::new("usr"), Inode::Directory(Box::new(tree.root.clone())));

    FileSystem {
        root,
        leaves: tree.leaves.clone(),
    }
}
