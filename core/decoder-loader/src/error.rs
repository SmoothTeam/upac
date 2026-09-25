// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;

use mime::FromStrError as MimeParseError;
use toml::de::Error as TomlError;

use upac_abi::error::ErrorKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecoderError {
    Load,
    Symbol,
    AbiMismatch { got: u32, expected: u32 },
    Failed(i32),
    InvalidResponse,
    Io(IoErrorKind),
    Manifest,
    DuplicateFormat(String),
    UnknownFormat(String),
    InvalidMimeType,
    NoDecoders,
}

impl From<ErrorKind> for DecoderError {
    fn from(_: ErrorKind) -> Self {
        DecoderError::InvalidResponse
    }
}

impl From<IoError> for DecoderError {
    fn from(error: IoError) -> Self {
        DecoderError::Io(error.kind())
    }
}

impl From<TomlError> for DecoderError {
    fn from(_: TomlError) -> Self {
        DecoderError::Manifest
    }
}

impl From<MimeParseError> for DecoderError {
    fn from(_: MimeParseError) -> Self {
        DecoderError::InvalidMimeType
    }
}

impl From<DecoderError> for ErrorKind {
    fn from(error: DecoderError) -> Self {
        match error {
            DecoderError::Load => ErrorKind::NotFound,
            DecoderError::Symbol => ErrorKind::AbiMismatch,
            DecoderError::AbiMismatch { .. } => ErrorKind::AbiMismatch,
            DecoderError::Failed(_) => ErrorKind::Unexpected,
            DecoderError::InvalidResponse => ErrorKind::InvalidEntry,
            DecoderError::Io(_) => ErrorKind::ReadFailed,
            DecoderError::Manifest => ErrorKind::InvalidEntry,
            DecoderError::DuplicateFormat(_) => ErrorKind::InvalidEntry,
            DecoderError::UnknownFormat(_) => ErrorKind::NotFound,
            DecoderError::InvalidMimeType => ErrorKind::InvalidEntry,
            DecoderError::NoDecoders => ErrorKind::NotFound,
        }
    }
}
