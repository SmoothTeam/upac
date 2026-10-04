// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::any::Any;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use efivar::Error as EfivarError;

use upac_types::booter::BootError;
use upac_types::error::ErrorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UkiError {
    EfiUnavailable,
    PermissionDenied,
    EntryNotFound,
    NoFreeBootId,
    InvalidRequest,
    Unexpected,
}

impl From<EfivarError> for UkiError {
    fn from(error: EfivarError) -> Self {
        match error {
            EfivarError::PermissionDenied { .. } => UkiError::PermissionDenied,
            _ => UkiError::Unexpected,
        }
    }
}

impl From<IoError> for UkiError {
    fn from(error: IoError) -> Self {
        match error.kind() {
            IoErrorKind::NotFound => UkiError::EntryNotFound,
            IoErrorKind::PermissionDenied => UkiError::PermissionDenied,
            _ => UkiError::Unexpected,
        }
    }
}

impl From<Box<dyn Any + Send + 'static>> for UkiError {
    fn from(_: Box<dyn Any + Send + 'static>) -> Self {
        UkiError::EfiUnavailable
    }
}

impl From<ErrorKind> for UkiError {
    fn from(error: ErrorKind) -> Self {
        match error {
            ErrorKind::PermissionDenied => UkiError::PermissionDenied,
            _ => UkiError::InvalidRequest,
        }
    }
}

impl From<UkiError> for BootError {
    fn from(error: UkiError) -> Self {
        match error {
            UkiError::EfiUnavailable => BootError::EfiUnavailable,
            UkiError::PermissionDenied => BootError::PermissionDenied,
            UkiError::EntryNotFound => BootError::EntryNotFound,
            UkiError::NoFreeBootId => BootError::NoFreeBootId,
            UkiError::InvalidRequest => BootError::InvalidRequest,
            UkiError::Unexpected => BootError::Unexpected,
        }
    }
}
