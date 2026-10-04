// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::Error as IoError;
use std::str::Utf8Error;

use serde::{Deserialize, Serialize};

use upac_abi::package::CPackageTrigger;
use upac_abi::types::{COwned, CSlice};

use upac_macro::{CEnum, CTryToRust, RustToC};

use crate::error::ErrorKind;

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
