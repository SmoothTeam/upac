// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::File;
use std::io::{Error as IoError, Write};
use std::os::fd::{FromRawFd, IntoRawFd};
use std::str::Utf8Error;

use upac_macro::CEnum;

use crate::error::ErrorKind;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, CEnum)]
pub enum DecodeError {
    InvalidRequest = 1,
    Io = 2,
    ChecksumMismatch = 3,
    UnsupportedFormat = 4,
    MissingMetadata = 5,
    MalformedMetadata = 6,
    InvalidUtf8 = 7,
    Cancelled = 8,
}

impl From<IoError> for DecodeError {
    fn from(_: IoError) -> Self {
        DecodeError::Io
    }
}

impl From<Utf8Error> for DecodeError {
    fn from(_: Utf8Error) -> Self {
        DecodeError::InvalidRequest
    }
}

impl From<ErrorKind> for DecodeError {
    fn from(_: ErrorKind) -> Self {
        DecodeError::InvalidRequest
    }
}

impl From<DecodeError> for ErrorKind {
    fn from(error: DecodeError) -> Self {
        match error {
            DecodeError::InvalidRequest => ErrorKind::InvalidEntry,
            DecodeError::Io => ErrorKind::ReadFailed,
            DecodeError::ChecksumMismatch => ErrorKind::InvalidEntry,
            DecodeError::UnsupportedFormat => ErrorKind::InvalidEntry,
            DecodeError::MissingMetadata => ErrorKind::InvalidEntry,
            DecodeError::MalformedMetadata => ErrorKind::InvalidEntry,
            DecodeError::InvalidUtf8 => ErrorKind::InvalidEntry,
            DecodeError::Cancelled => ErrorKind::Cancelled,
        }
    }
}

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
