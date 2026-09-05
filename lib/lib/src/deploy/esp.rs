// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::PathBuf;

use rsmount::tables::MountInfo;

use crate::deploy::error::SysrootError;
use crate::layout::boot::{ESP_MOUNT_FALLBACK, ESP_MOUNT_PRIMARY};

pub fn find_esp_mount() -> Result<PathBuf, SysrootError> {
    let mut mount_table = MountInfo::new()?;
    mount_table.import_mountinfo()?;

    for candidate_for_mount in [ESP_MOUNT_PRIMARY, ESP_MOUNT_FALLBACK] {
        if mount_table.find_target(candidate_for_mount).is_some() {
            return Ok(PathBuf::from(candidate_for_mount));
        }
    }

    Err(SysrootError::EspNotFound)
}
