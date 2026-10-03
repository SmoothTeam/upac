// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{Permissions, metadata, read, remove_file};
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Write as IoWrite};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use tempfile::NamedTempFile;

use upac_abi::error::ErrorKind;

use super::layout::fs::NEW_FILE_MODE;

struct PreviousFile {
    content: Vec<u8>,
    permissions: Permissions,
}

pub struct WrittenFile {
    path: PathBuf,
    previous: Option<PreviousFile>,
}

impl WrittenFile {
    pub fn write(path: &Path, content: &[u8]) -> Result<Self, IoError> {
        let previous = match read(path) {
            Ok(previous_content) => Some(PreviousFile {
                content: previous_content,
                permissions: metadata(path)?.permissions(),
            }),
            Err(error) if error.kind() == IoErrorKind::NotFound => None,
            Err(error) => return Err(error),
        };

        let permissions = match &previous {
            Some(previous) => previous.permissions.clone(),
            None => Permissions::from_mode(NEW_FILE_MODE),
        };

        let written = WrittenFile {
            path: path.to_owned(),
            previous,
        };
        written.atomic_write(content, permissions)?;

        Ok(written)
    }

    pub fn restore(&self) -> Result<(), ErrorKind> {
        self.restore_previous().map_err(|error| match error.kind() {
            IoErrorKind::NotFound => ErrorKind::NotFound,
            IoErrorKind::PermissionDenied => ErrorKind::PermissionDenied,
            _ => ErrorKind::Unexpected,
        })
    }

    fn restore_previous(&self) -> Result<(), IoError> {
        match &self.previous {
            Some(previous) => self.atomic_write(&previous.content, previous.permissions.clone()),
            None => match remove_file(&self.path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == IoErrorKind::NotFound => Ok(()),
                Err(error) => Err(error),
            },
        }
    }

    fn atomic_write(&self, content: &[u8], permissions: Permissions) -> Result<(), IoError> {
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));

        let mut tmp_file = NamedTempFile::new_in(parent)?;
        tmp_file.write_all(content)?;
        tmp_file.as_file().set_permissions(permissions)?;
        tmp_file.as_file().sync_all()?;
        tmp_file.persist(&self.path).map_err(|error| error.error)?;

        Ok(())
    }
}
