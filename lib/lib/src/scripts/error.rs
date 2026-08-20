// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;
use std::str::Utf8Error;

use toml::de::Error as TomlError;

use upac_abi::error::ErrorKind;

use upac_pki::error::PkiError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookError {
    Parse,
    InvalidTrigger,
    NoTrigger,
    Io(IoErrorKind),
    Encoding,
    MalformedSignature,
    InvalidSignature,
    TriggerConflict(String),
}

impl From<TomlError> for HookError {
    fn from(_: TomlError) -> Self {
        HookError::Parse
    }
}

impl From<IoError> for HookError {
    fn from(error: IoError) -> Self {
        HookError::Io(error.kind())
    }
}

impl From<Utf8Error> for HookError {
    fn from(_: Utf8Error) -> Self {
        HookError::Encoding
    }
}

impl From<PkiError> for HookError {
    fn from(error: PkiError) -> Self {
        match error {
            PkiError::Malformed => HookError::MalformedSignature,
            PkiError::InvalidSignature => HookError::InvalidSignature,
            PkiError::Generation => HookError::Parse,
        }
    }
}

impl From<HookError> for ErrorKind {
    fn from(error: HookError) -> Self {
        match error {
            HookError::Parse => ErrorKind::InvalidEntry,
            HookError::InvalidTrigger => ErrorKind::InvalidEntry,
            HookError::NoTrigger => ErrorKind::InvalidEntry,
            HookError::Io(_) => ErrorKind::ReadFailed,
            HookError::Encoding => ErrorKind::InvalidEntry,
            HookError::MalformedSignature => ErrorKind::InvalidEntry,
            HookError::InvalidSignature => ErrorKind::InvalidEntry,
            HookError::TriggerConflict(_) => ErrorKind::InvalidEntry,
        }
    }
}
