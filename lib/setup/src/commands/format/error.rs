// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use anyhow::Error as AnyhowError;

use upac::errors::CommonError;
use upac::lock::LockError;

use upac_abi::error::ErrorKind;

use crate::commands::partition::error::PartitionError;
use crate::wipe::WipeError;

#[cfg(test)]
#[path = "../../../tests/inline/format_error.rs"]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    Common(CommonError),
    Io(IoErrorKind),
    Partition(PartitionError),
    NotEspPartition,
    InvalidFormatParams,
    DeviceNotEmpty,
    WipeFailed,
    MkfsFailed,
    Unexpected,
}

impl From<CommonError> for FormatError {
    fn from(error: CommonError) -> Self {
        FormatError::Common(error)
    }
}

impl From<LockError> for FormatError {
    fn from(error: LockError) -> Self {
        FormatError::Common(CommonError::Lock(error))
    }
}

impl From<IoError> for FormatError {
    fn from(error: IoError) -> Self {
        FormatError::Io(error.kind())
    }
}

impl From<PartitionError> for FormatError {
    fn from(error: PartitionError) -> Self {
        FormatError::Partition(error)
    }
}

impl From<WipeError> for FormatError {
    fn from(error: WipeError) -> Self {
        match error {
            WipeError::Io(kind) => FormatError::Io(kind),
            WipeError::DeviceNotEmpty => FormatError::DeviceNotEmpty,
            WipeError::WipeFailed => FormatError::WipeFailed,
        }
    }
}

impl From<AnyhowError> for FormatError {
    fn from(_: AnyhowError) -> Self {
        FormatError::MkfsFailed
    }
}

impl From<FormatError> for ErrorKind {
    fn from(error: FormatError) -> Self {
        match error {
            FormatError::Common(common_error) => common_error.into(),
            FormatError::Io(kind) => match kind {
                IoErrorKind::NotFound => ErrorKind::NotFound,
                IoErrorKind::PermissionDenied => ErrorKind::PermissionDenied,
                IoErrorKind::AlreadyExists => ErrorKind::AlreadyExists,
                _ => ErrorKind::Unexpected,
            },
            FormatError::Partition(partition_error) => partition_error.into(),
            FormatError::NotEspPartition => ErrorKind::InvalidEntry,
            FormatError::InvalidFormatParams => ErrorKind::InvalidEntry,
            FormatError::DeviceNotEmpty => ErrorKind::AlreadyExists,
            FormatError::WipeFailed => ErrorKind::WriteFailed,
            FormatError::MkfsFailed => ErrorKind::WriteFailed,
            FormatError::Unexpected => ErrorKind::Unexpected,
        }
    }
}
