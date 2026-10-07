// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::{Path, PathBuf};

use upac_types::error::ErrorKind;
use upac_types::request::partition::PartitionKind;

use crate::gpt::Disk;

#[cfg(test)]
#[path = "../../tests/inline/commands.rs"]
mod tests;

pub mod add;
pub mod deploy;
pub mod format;
pub mod import;
pub mod init;
pub mod kernel;

pub(crate) fn root_partition(disk: Option<&str>, deploy_device: Option<&str>) -> Result<PathBuf, ErrorKind> {
    match (disk, deploy_device) {
        (Some(disk), None) => Ok(Disk::open(Path::new(disk))?
            .find(PartitionKind::Root)?
            .path()
            .to_path_buf()),
        (None, Some(deploy_device)) => Ok(PathBuf::from(deploy_device)),
        _ => Err(ErrorKind::InvalidEntry),
    }
}
