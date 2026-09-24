// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use gptman::Error as GptError;
use gptman::linux::BlockError as GptBlockError;

use nix::errno::Errno;

use upac::errors::CommonError;
use upac::lock::LockError;

use upac_abi::error::ErrorKind;

use crate::wipe::WipeError;

#[cfg(test)]
#[path = "../../../tests/inline/partition_error.rs"]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartitionError {
    Common(CommonError),
    Io(IoErrorKind),
    NotBlockDevice,
    DeviceNotEmpty,
    WipeFailed,
    TableNotFound,
    NoFreeSlot,
    NoSpaceLeft,
    InvalidSize,
    LabelTooLong,
    InvalidPartitionLayout,
    RereadFailed(Errno),
    PartitionNotReady,
    Unexpected,
}

impl From<CommonError> for PartitionError {
    fn from(error: CommonError) -> Self {
        PartitionError::Common(error)
    }
}

impl From<LockError> for PartitionError {
    fn from(error: LockError) -> Self {
        PartitionError::Common(CommonError::Lock(error))
    }
}

impl From<IoError> for PartitionError {
    fn from(error: IoError) -> Self {
        PartitionError::Io(error.kind())
    }
}

impl From<WipeError> for PartitionError {
    fn from(error: WipeError) -> Self {
        match error {
            WipeError::Io(kind) => PartitionError::Io(kind),
            WipeError::DeviceNotEmpty => PartitionError::DeviceNotEmpty,
            WipeError::WipeFailed => PartitionError::WipeFailed,
        }
    }
}

impl From<GptError> for PartitionError {
    fn from(error: GptError) -> Self {
        match error {
            GptError::Io(io_error) => PartitionError::Io(io_error.kind()),
            GptError::InvalidSignature | GptError::ReadError(_, _) => PartitionError::TableNotFound,
            GptError::NoSpaceLeft => PartitionError::NoSpaceLeft,
            GptError::InvalidPartitionBoundaries => PartitionError::InvalidPartitionLayout,
            _ => PartitionError::Unexpected,
        }
    }
}

impl From<GptBlockError> for PartitionError {
    fn from(error: GptBlockError) -> Self {
        match error {
            GptBlockError::Metadata(io_error) => PartitionError::Io(io_error.kind()),
            GptBlockError::NotBlock => PartitionError::NotBlockDevice,
            GptBlockError::RereadTable(errno) => PartitionError::RereadFailed(errno),
            _ => PartitionError::Unexpected,
        }
    }
}

impl From<PartitionError> for ErrorKind {
    fn from(error: PartitionError) -> Self {
        match error {
            PartitionError::Common(common_error) => common_error.into(),
            PartitionError::Io(kind) => match kind {
                IoErrorKind::NotFound => ErrorKind::NotFound,
                IoErrorKind::PermissionDenied => ErrorKind::PermissionDenied,
                IoErrorKind::AlreadyExists => ErrorKind::AlreadyExists,
                _ => ErrorKind::Unexpected,
            },
            PartitionError::NotBlockDevice => ErrorKind::InvalidEntry,
            PartitionError::DeviceNotEmpty => ErrorKind::AlreadyExists,
            PartitionError::WipeFailed => ErrorKind::WriteFailed,
            PartitionError::TableNotFound => ErrorKind::NotFound,
            PartitionError::NoFreeSlot => ErrorKind::NoSpaceLeft,
            PartitionError::NoSpaceLeft => ErrorKind::NoSpaceLeft,
            PartitionError::InvalidSize => ErrorKind::InvalidEntry,
            PartitionError::LabelTooLong => ErrorKind::InvalidEntry,
            PartitionError::InvalidPartitionLayout => ErrorKind::InvalidEntry,
            PartitionError::RereadFailed(_) => ErrorKind::ReadFailed,
            PartitionError::PartitionNotReady => ErrorKind::NotInitialized,
            PartitionError::Unexpected => ErrorKind::Unexpected,
        }
    }
}
