// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::error::ErrorKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootError {
    NoBootResource,
    AmbiguousBootResource,
    BootEntryNotFound,
    AccessDenied,
    Unexpected,
}

impl From<anyhow::Error> for BootError {
    fn from(_: anyhow::Error) -> Self {
        BootError::Unexpected
    }
}

impl From<efivar::Error> for BootError {
    fn from(error: efivar::Error) -> Self {
        match error {
            efivar::Error::PermissionDenied { .. } => BootError::AccessDenied,
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
            BootError::AccessDenied => ErrorKind::PermissionDenied,
            BootError::Unexpected => ErrorKind::Unexpected,
        }
    }
}
