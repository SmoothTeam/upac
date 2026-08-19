// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::any::Any;

use efivar::Error as EfivarError;

use upac_abi::error::ErrorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UkiError {
    EfiUnavailable,
    PermissionDenied,
    EntryNotFound,
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

impl From<Box<dyn Any + Send + 'static>> for UkiError {
    fn from(_: Box<dyn Any + Send + 'static>) -> Self {
        UkiError::EfiUnavailable
    }
}

impl From<UkiError> for ErrorKind {
    fn from(error: UkiError) -> Self {
        match error {
            UkiError::EfiUnavailable => ErrorKind::NotInitialized,
            UkiError::PermissionDenied => ErrorKind::PermissionDenied,
            UkiError::EntryNotFound => ErrorKind::NotFound,
            UkiError::InvalidRequest => ErrorKind::InvalidEntry,
            UkiError::Unexpected => ErrorKind::Unexpected,
        }
    }
}
