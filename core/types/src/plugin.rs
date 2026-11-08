// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::File;
use std::io::{Error as IoError, Write};
use std::os::fd::{FromRawFd, IntoRawFd};

use crate::error::ErrorKind;

const PLUGIN_SUCCESS_STATUS: i32 = 0;

pub struct ObjectFile(File);

impl ObjectFile {
    /// # Safety
    /// `fd` must be an open file descriptor that nothing else owns or closes: the returned value
    /// takes it over and closes it when dropped.
    pub unsafe fn from_raw(fd: i32) -> Result<Self, ErrorKind> {
        if fd < 0 {
            return Err(ErrorKind::InvalidEntry);
        }

        Ok(Self(unsafe { File::from_raw_fd(fd) }))
    }

    pub fn into_file(self) -> File {
        self.0
    }
}

impl From<File> for ObjectFile {
    fn from(file: File) -> Self {
        Self(file)
    }
}

impl From<ObjectFile> for i32 {
    fn from(file: ObjectFile) -> Self {
        file.0.into_raw_fd()
    }
}

impl Write for ObjectFile {
    fn write(&mut self, buffer: &[u8]) -> Result<usize, IoError> {
        self.0.write(buffer)
    }

    fn flush(&mut self) -> Result<(), IoError> {
        self.0.flush()
    }
}

pub fn plugin_status<StatusError: Into<i32>>(result: Result<(), StatusError>) -> i32 {
    match result {
        Ok(()) => PLUGIN_SUCCESS_STATUS,
        Err(error) => error.into(),
    }
}

pub fn plugin_result<StatusError: TryFrom<i32>>(status: i32) -> Result<(), Option<StatusError>> {
    if status == PLUGIN_SUCCESS_STATUS {
        return Ok(());
    }

    Err(StatusError::try_from(status).ok())
}
