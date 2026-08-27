// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::Path;

use composefs::repository::Repository;
use composefs::tree::FileSystem;
use composefs_boot::bootloader::{BootEntry, get_boot_resources};
use composefs_boot::cmdline::ComposefsCmdline;
use composefs_boot::write_boot::write_boot_simple;

use self::error::BootError;

use crate::composefs::repository::ObjectID;
use crate::layout::boot::UPAC_TO_SLOT;

pub mod error;

/// Writes the boot entry for `digest` onto `boot_partition` and returns the entry name actually
/// used. A UKI-direct image (`BootEntry::Type2`) is always staged under the fixed
/// [`UPAC_TO_SLOT`] stem — the corresponding UEFI `Boot####` entry is pre-registered once,
/// outside this pipeline, and this call only overwrites that file's content. A BLS-style image
/// (`BootEntry::Type1`) uses `prefix_digest` itself as the entry name — content-addressed, no
/// fixed slot needed, since systemd-boot/grub/refind all rescan their entries directory fresh.
pub fn write_boot_entry(
    repository: &Repository<ObjectID>, tree: &FileSystem<ObjectID>, digest: ObjectID, boot_partition: &Path,
    prefix_digest: &str,
) -> Result<String, BootError> {
    let mut entries = get_boot_resources(tree, repository)?;

    if entries.len() > 1 {
        return Err(BootError::AmbiguousBootResource);
    }
    let entry = entries.pop().ok_or(BootError::NoBootResource)?;

    let entry_name = match entry {
        BootEntry::Type1(_) => prefix_digest.to_owned(),
        BootEntry::Type2(_) => UPAC_TO_SLOT.to_owned(),
        BootEntry::UsrLibModulesVmLinuz(_) => return Err(BootError::UnsupportedBootResource),
    };

    let karg = ComposefsCmdline::new_v2(digest, false);
    write_boot_simple(repository, entry, &karg, boot_partition, None, Some(&entry_name), &[])?;

    Ok(entry_name)
}
