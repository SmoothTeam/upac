// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::ErrorKind as IoErrorKind;
use std::path::Path;
use std::process::{Command, Output};

use upac_types::error::ErrorKind;

use super::layout::mkfs::WIPEFS_BIN;

pub(crate) struct WipeTarget<'target> {
    pub device_path: &'target Path,
}

impl WipeTarget<'_> {
    pub fn wipe_or_refuse(&self, force_wipe: bool) -> Result<(), ErrorKind> {
        if force_wipe {
            return self.wipe();
        }

        self.ensure_empty()
    }

    fn wipe(&self) -> Result<(), ErrorKind> {
        let output = Self::run_wipefs(Command::new(WIPEFS_BIN).arg("-a").arg(self.device_path.as_os_str()))?;

        if !output.status.success() {
            return Err(ErrorKind::ToolFailed);
        }

        Ok(())
    }

    fn ensure_empty(&self) -> Result<(), ErrorKind> {
        let output = Self::run_wipefs(Command::new(WIPEFS_BIN).arg(self.device_path.as_os_str()))?;

        if !output.status.success() {
            return Err(ErrorKind::ToolFailed);
        }

        if !output.stdout.is_empty() {
            return Err(ErrorKind::AlreadyExists);
        }

        Ok(())
    }

    fn run_wipefs(command: &mut Command) -> Result<Output, ErrorKind> {
        command.output().map_err(|error| match error.kind() {
            IoErrorKind::NotFound => ErrorKind::ToolNotInstalled,
            _ => error.into(),
        })
    }
}
