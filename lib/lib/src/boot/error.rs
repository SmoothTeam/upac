// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::any::Any;

use anyhow::Error as AnyhowError;

use efivar::Error as EfivarError;

use uuid::Error as UuidError;

use upac_abi::error::ErrorKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootError {
    NoBootResource,
    AmbiguousBootResource,
    BootEntryNotFound,
    EfiNotAvailable,
    AccessDenied,
    Unexpected,
}

impl From<AnyhowError> for BootError {
    fn from(_: AnyhowError) -> Self {
        BootError::Unexpected
    }
}

impl From<UuidError> for BootError {
    fn from(_: UuidError) -> Self {
        BootError::Unexpected
    }
}

impl From<Box<dyn Any + Send + 'static>> for BootError {
    fn from(_: Box<dyn Any + Send + 'static>) -> Self {
        BootError::EfiNotAvailable
    }
}

impl From<EfivarError> for BootError {
    fn from(error: EfivarError) -> Self {
        match error {
            EfivarError::PermissionDenied { .. } => BootError::AccessDenied,
            _ => BootError::Unexpected,
        }
    }
}

impl From<BootError> for ErrorKind {
    fn from(error: BootError) -> Self {
        match error {
            BootError::NoBootResource => ErrorKind::NotFound,
            BootError::AmbiguousBootResource => ErrorKind::InvalidEntry,
            BootError::BootEntryNotFound => ErrorKind::NotFound,
            BootError::EfiNotAvailable => ErrorKind::NotInitialized,
            BootError::AccessDenied => ErrorKind::PermissionDenied,
            BootError::Unexpected => ErrorKind::Unexpected,
        }
    }
}
