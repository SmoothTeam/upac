// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::any::Any;

use efivar::Error as EfivarError;

use uuid::Error as UuidError;

use upac_abi::error::ErrorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefindError {
    EfiUnavailable,
    PermissionDenied,
    InvalidRequest,
    Unexpected,
}

impl From<EfivarError> for RefindError {
    fn from(error: EfivarError) -> Self {
        match error {
            EfivarError::PermissionDenied { .. } => RefindError::PermissionDenied,
            _ => RefindError::Unexpected,
        }
    }
}

impl From<UuidError> for RefindError {
    fn from(_: UuidError) -> Self {
        RefindError::Unexpected
    }
}

impl From<Box<dyn Any + Send + 'static>> for RefindError {
    fn from(_: Box<dyn Any + Send + 'static>) -> Self {
        RefindError::EfiUnavailable
    }
}

impl From<RefindError> for ErrorKind {
    fn from(error: RefindError) -> Self {
        match error {
            RefindError::EfiUnavailable => ErrorKind::NotInitialized,
            RefindError::PermissionDenied => ErrorKind::PermissionDenied,
            RefindError::InvalidRequest => ErrorKind::InvalidEntry,
            RefindError::Unexpected => ErrorKind::Unexpected,
        }
    }
}
