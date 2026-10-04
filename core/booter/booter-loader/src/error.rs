// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;

use rsmount::errors::MountInfoError;

use toml::de::Error as TomlError;

use upac_types::booter::BootError;
use upac_types::error::ErrorKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootPluginError {
    Load,
    Symbol,
    AbiMismatch { got: u32, expected: u32 },
    Failed(BootError),
    InvalidResponse,
    Io(IoErrorKind),
    Manifest,
    DuplicateName(String),
    UnknownName(String),
    NoClaimant,
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
            BootPluginError::Failed(error) => error.into(),
            BootPluginError::InvalidResponse => ErrorKind::InvalidEntry,
            BootPluginError::Io(_) => ErrorKind::ReadFailed,
            BootPluginError::Manifest => ErrorKind::InvalidEntry,
            BootPluginError::DuplicateName(_) => ErrorKind::InvalidEntry,
            BootPluginError::UnknownName(_) => ErrorKind::NotFound,
            BootPluginError::NoClaimant => ErrorKind::NotFound,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EspError {
    MountInfoUnavailable,
    NotFound,
}

impl From<MountInfoError> for EspError {
    fn from(_: MountInfoError) -> Self {
        EspError::MountInfoUnavailable
    }
}

impl From<EspError> for ErrorKind {
    fn from(error: EspError) -> Self {
        match error {
            EspError::MountInfoUnavailable => ErrorKind::Unexpected,
            EspError::NotFound => ErrorKind::NotFound,
        }
    }
}
