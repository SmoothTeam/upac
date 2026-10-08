// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{create_dir, remove_dir_all, rename};
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::path::PathBuf;

pub struct SetAsideEtc {
    upper_dir: PathBuf,
    set_aside_dir: PathBuf,
}

impl SetAsideEtc {
    pub(crate) fn new(upper_dir: PathBuf, set_aside_dir: PathBuf) -> Result<Self, IoError> {
        match remove_dir_all(&set_aside_dir) {
            Ok(()) => {}
            Err(error) if error.kind() == IoErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }

        rename(&upper_dir, &set_aside_dir)?;
        if let Err(error) = create_dir(&upper_dir) {
            rename(&set_aside_dir, &upper_dir)?;
            return Err(error);
        }

        Ok(Self {
            upper_dir,
            set_aside_dir,
        })
    }

    pub fn restore(&self) -> Result<(), IoError> {
        remove_dir_all(&self.upper_dir)?;
        rename(&self.set_aside_dir, &self.upper_dir)
    }

    pub fn discard(self) -> Result<(), IoError> {
        remove_dir_all(&self.set_aside_dir)
    }
}
