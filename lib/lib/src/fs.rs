// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Write as IoWrite};
use std::path::{Path, PathBuf};

use upac_abi::error::ErrorKind;

use crate::orchestrator::stage::{RollbackGuard, StageResult};

pub fn atomic_write(path: &Path, content: &[u8]) -> Result<(), IoError> {
    let mut tmp_name = OsString::from(path.as_os_str());
    tmp_name.push(".tmp");
    let tmp_path = PathBuf::from(tmp_name);

    let mut tmp_file = File::create(&tmp_path)?;
    tmp_file.write_all(content)?;
    tmp_file.sync_all()?;

    fs::rename(&tmp_path, path)
}

/// A file written via [`atomic_write`], remembering its previous content (if any) so a group of
/// writes can be undone as a unit — push each successfully written file into a `Vec<WrittenFile>`
/// and call `.rollback()` on it (manually, on a later write's failure, or via the orchestrator's
/// own [`RollbackGuard`] machinery if a later stage fails).
pub struct WrittenFile {
    path: PathBuf,
    previous: Option<Vec<u8>>,
}

impl WrittenFile {
    pub fn write(path: &Path, content: &[u8]) -> Result<Self, IoError> {
        let previous = fs::read(path).ok();
        atomic_write(path, content)?;

        Ok(WrittenFile {
            path: path.to_owned(),
            previous,
        })
    }

    fn restore(&self) -> Result<(), IoError> {
        match &self.previous {
            Some(bytes) => atomic_write(&self.path, bytes),
            None => match fs::remove_file(&self.path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == IoErrorKind::NotFound => Ok(()),
                Err(error) => Err(error),
            },
        }
    }
}

impl RollbackGuard for Vec<WrittenFile> {
    fn new_none(_result: StageResult) -> Self {
        Vec::new()
    }

    fn rollback(&mut self) -> Result<(), ErrorKind> {
        while let Some(file) = self.pop() {
            file.restore().map_err(|error| match error.kind() {
                IoErrorKind::NotFound => ErrorKind::NotFound,
                IoErrorKind::PermissionDenied => ErrorKind::PermissionDenied,
                _ => ErrorKind::Unexpected,
            })?;
        }

        Ok(())
    }

    fn result(&self) -> StageResult {
        StageResult::Advance
    }
}
