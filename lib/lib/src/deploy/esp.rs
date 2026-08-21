// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::path::PathBuf;

use rsmount::tables::MountInfo;

use crate::deploy::error::SysrootError;
use crate::layout::boot::{ESP_MOUNT_FALLBACK, ESP_MOUNT_PRIMARY};

pub fn find_esp_mount() -> Result<PathBuf, SysrootError> {
    let mut table = MountInfo::new()?;
    table.import_mountinfo()?;

    for candidate in [ESP_MOUNT_PRIMARY, ESP_MOUNT_FALLBACK] {
        if table.find_target(candidate).is_some() {
            return Ok(PathBuf::from(candidate));
        }
    }

    Err(SysrootError::EspNotFound)
}
