// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs::File;
use std::io::{Error as IoError, Write};
use std::os::fd::{FromRawFd, IntoRawFd};
use std::str::{FromStr, Utf8Error};

use serde::{Deserialize, Serialize};

use upac_abi::CPackageTrigger;
use upac_abi::types::{COwned, CSlice};

use upac_macro::{CEnum, CTryToRust, RustToC};

use crate::error::ErrorKind;
use crate::{DecodedPackageMeta, PackageChecksum};

const PLUGIN_SUCCESS_STATUS: i32 = 0;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, CEnum)]
pub enum TriggerPosition {
    PreInstall = 0,
    PostInstall = 1,
    PreUpgrade = 2,
    PostUpgrade = 3,
    PreRemove = 4,
    PostRemove = 5,
}

impl TriggerPosition {
    pub const ALL: [TriggerPosition; 6] = [
        TriggerPosition::PreInstall,
        TriggerPosition::PostInstall,
        TriggerPosition::PreUpgrade,
        TriggerPosition::PostUpgrade,
        TriggerPosition::PreRemove,
        TriggerPosition::PostRemove,
    ];
}

impl Display for TriggerPosition {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_str(match self {
            TriggerPosition::PreInstall => "pre-install",
            TriggerPosition::PostInstall => "post-install",
            TriggerPosition::PreUpgrade => "pre-upgrade",
            TriggerPosition::PostUpgrade => "post-upgrade",
            TriggerPosition::PreRemove => "pre-remove",
            TriggerPosition::PostRemove => "post-remove",
        })
    }
}

impl FromStr for TriggerPosition {
    type Err = DecodeError;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        TriggerPosition::ALL
            .into_iter()
            .find(|position| position.to_string() == name)
            .ok_or(DecodeError::MalformedMetadata)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CTryToRust, RustToC)]
pub struct PackageTrigger {
    pub position: TriggerPosition,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageTriggers {
    pub format: String,
    pub triggers: Vec<PackageTrigger>,
}

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

pub trait DecodeMeta {
    fn decode(&self, checksum: PackageChecksum) -> Result<DecodedPackageMeta, DecodeError>;
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
