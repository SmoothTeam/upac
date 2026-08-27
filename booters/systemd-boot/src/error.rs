// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::any::Any;

use efivar::Error as EfivarError;

use uuid::Error as UuidError;

use upac_abi::error::ErrorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlsError {
    EfiUnavailable,
    PermissionDenied,
    InvalidRequest,
    Unexpected,
}

impl From<EfivarError> for BlsError {
    fn from(error: EfivarError) -> Self {
        match error {
            EfivarError::PermissionDenied { .. } => BlsError::PermissionDenied,
            _ => BlsError::Unexpected,
        }
    }
}

impl From<UuidError> for BlsError {
    fn from(_: UuidError) -> Self {
        BlsError::Unexpected
    }
}

impl From<Box<dyn Any + Send + 'static>> for BlsError {
    fn from(_: Box<dyn Any + Send + 'static>) -> Self {
        BlsError::EfiUnavailable
    }
}

impl From<BlsError> for ErrorKind {
    fn from(error: BlsError) -> Self {
        match error {
            BlsError::EfiUnavailable => ErrorKind::NotInitialized,
            BlsError::PermissionDenied => ErrorKind::PermissionDenied,
            BlsError::InvalidRequest => ErrorKind::InvalidEntry,
            BlsError::Unexpected => ErrorKind::Unexpected,
        }
    }
}
