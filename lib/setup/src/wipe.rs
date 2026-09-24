// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::path::Path;
use std::process::Command;

use super::layout::mkfs::WIPEFS_BIN;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WipeError {
    Io(IoErrorKind),
    DeviceNotEmpty,
    WipeFailed,
}

impl From<IoError> for WipeError {
    fn from(error: IoError) -> Self {
        WipeError::Io(error.kind())
    }
}

pub(crate) struct WipeTarget<'target> {
    pub device_path: &'target Path,
}

impl WipeTarget<'_> {
    pub fn wipe_or_refuse(&self, force_wipe: bool) -> Result<(), WipeError> {
        if force_wipe {
            return self.wipe();
        }

        self.ensure_empty()
    }

    fn wipe(&self) -> Result<(), WipeError> {
        let status = Command::new(WIPEFS_BIN)
            .arg("-a")
            .arg(self.device_path.as_os_str())
            .status()?;

        if !status.success() {
            return Err(WipeError::WipeFailed);
        }

        Ok(())
    }

    fn ensure_empty(&self) -> Result<(), WipeError> {
        let output = Command::new(WIPEFS_BIN).arg(self.device_path.as_os_str()).output()?;

        if !output.status.success() {
            return Err(WipeError::WipeFailed);
        }

        if !output.stdout.is_empty() {
            return Err(WipeError::DeviceNotEmpty);
        }

        Ok(())
    }
}
