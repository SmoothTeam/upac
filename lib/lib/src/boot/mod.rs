// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::path::Path;

use composefs::repository::Repository;
use composefs::tree::FileSystem;
use composefs_boot::bootloader::get_boot_resources;
use composefs_boot::cmdline::ComposefsCmdline;
use composefs_boot::write_boot::write_boot_simple;

use self::error::BootError;

use crate::composefs::repository::ObjectID;

pub mod error;

pub fn write_boot_entry(
    repository: &Repository<ObjectID>, tree: &FileSystem<ObjectID>, digest: ObjectID, boot_partition: &Path,
    entry_name: &str,
) -> Result<(), BootError> {
    let mut entries = get_boot_resources(tree, repository)?;

    if entries.len() > 1 {
        return Err(BootError::AmbiguousBootResource);
    }
    let entry = entries.pop().ok_or(BootError::NoBootResource)?;

    let karg = ComposefsCmdline::new_v2(digest, false);
    write_boot_simple(repository, entry, &karg, boot_partition, None, Some(entry_name), &[])?;

    Ok(())
}
