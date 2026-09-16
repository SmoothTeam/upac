// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsStr;
use std::path::Path;

use composefs::generic_tree::Stat;
use composefs::repository::Repository;
use composefs::tree::{Directory, FileSystem, Inode};

use composefs_boot::bootloader::{BootEntry, get_boot_resources};
use composefs_boot::cmdline::ComposefsCmdline;
use composefs_boot::write_boot::write_boot_simple;

use upac_abi::BootResourceKind;

use self::error::BootError;

use crate::composefs::repository::ObjectID;
use crate::layout::boot::UPAC_UKI_TO_SLOT;

pub mod error;

#[derive(Debug)]
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

pub fn write_boot_entry(
    repository: &Repository<ObjectID>, tree: &FileSystem<ObjectID>, digest: ObjectID, boot_partition: &Path,
    prefix_digest: &str, wanted: BootResourceKind,
) -> Result<WrittenBootEntry, BootError> {
    let rooted_tree = wrap_under_usr(tree);
    let entries = get_boot_resources(&rooted_tree, repository)?;

    if entries.is_empty() {
        return Err(BootError::NoBootResource);
    }

    let mut matching: Vec<_> = entries
        .into_iter()
        .filter(|entry| {
            matches!(
                (wanted, entry),
                (BootResourceKind::Uki, BootEntry::Type2(_))
                    | (
                        BootResourceKind::Bls,
                        BootEntry::Type1(_) | BootEntry::UsrLibModulesVmLinuz(_)
                    )
            )
        })
        .collect();

    if matching.len() > 1 {
        return Err(BootError::AmbiguousBootResource);
    }
    let entry = matching.pop().ok_or(BootError::UnsupportedBootResource)?;

    let written = match &entry {
        BootEntry::Type1(_) | BootEntry::UsrLibModulesVmLinuz(_) => WrittenBootEntry::Bls(prefix_digest.to_owned()),
        BootEntry::Type2(_) => WrittenBootEntry::Uki(UPAC_UKI_TO_SLOT.to_owned()),
    };

    let karg = ComposefsCmdline::new_v2(digest, false);
    write_boot_simple(
        repository,
        entry,
        &karg,
        boot_partition,
        None,
        Some(written.entry_name()),
        &[],
    )?;

    Ok(written)
}

fn wrap_under_usr(tree: &FileSystem<ObjectID>) -> FileSystem<ObjectID> {
    let mut root = Directory::new(Stat::uninitialized());
    root.insert(OsStr::new("usr"), Inode::Directory(Box::new(tree.root.clone())));

    FileSystem {
        root,
        leaves: tree.leaves.clone(),
    }
}
