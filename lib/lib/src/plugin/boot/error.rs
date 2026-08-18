// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;

use toml::de::Error as TomlError;

use upac_abi::error::ErrorKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootPluginError {
    Load,
    Symbol,
    AbiMismatch { got: u32, expected: u32 },
    Failed(i32),
    Io(IoErrorKind),
    Manifest,
    DuplicateName(String),
    UnknownName(String),
    NoClaimant,
    AmbiguousClaim,
}

impl From<IoError> for BootPluginError {
    fn from(error: IoError) -> Self {
        BootPluginError::Io(error.kind())
    }
}

impl From<TomlError> for BootPluginError {
    fn from(_: TomlError) -> Self {
        BootPluginError::Manifest
    }
}

impl From<BootPluginError> for ErrorKind {
    fn from(error: BootPluginError) -> Self {
        match error {
            BootPluginError::Load => ErrorKind::NotFound,
            BootPluginError::Symbol => ErrorKind::AbiMismatch,
            BootPluginError::AbiMismatch { .. } => ErrorKind::AbiMismatch,
            BootPluginError::Failed(_) => ErrorKind::Unexpected,
            BootPluginError::Io(_) => ErrorKind::ReadFailed,
            BootPluginError::Manifest => ErrorKind::InvalidEntry,
            BootPluginError::DuplicateName(_) => ErrorKind::InvalidEntry,
            BootPluginError::UnknownName(_) => ErrorKind::NotFound,
            BootPluginError::NoClaimant => ErrorKind::NotFound,
            BootPluginError::AmbiguousClaim => ErrorKind::InvalidEntry,
        }
    }
}
