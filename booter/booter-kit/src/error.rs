// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;

use toml::de::Error as TomlError;

use upac_types::booter::BootError;
use upac_types::error::ErrorKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BooterError {
    Load,
    Symbol,
    AbiMismatch { got: u32, expected: u32 },
    Failed(BootError),
    InvalidResponse,
    Io(IoErrorKind),
    Manifest,
}

impl From<BootError> for BooterError {
    fn from(error: BootError) -> Self {
        BooterError::Failed(error)
    }
}

impl From<IoError> for BooterError {
    fn from(error: IoError) -> Self {
        BooterError::Io(error.kind())
    }
}

impl From<TomlError> for BooterError {
    fn from(_: TomlError) -> Self {
        BooterError::Manifest
    }
}

impl From<BooterError> for ErrorKind {
    fn from(error: BooterError) -> Self {
        match error {
            BooterError::Load => ErrorKind::NotFound,
            BooterError::Symbol => ErrorKind::AbiMismatch,
            BooterError::AbiMismatch { .. } => ErrorKind::AbiMismatch,
            BooterError::Failed(error) => error.into(),
            BooterError::InvalidResponse => ErrorKind::InvalidEntry,
            BooterError::Io(_) => ErrorKind::ReadFailed,
            BooterError::Manifest => ErrorKind::InvalidEntry,
        }
    }
}
