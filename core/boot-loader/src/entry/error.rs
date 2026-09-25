// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use anyhow::Error as AnyhowError;

use upac_abi::error::ErrorKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootError {
    NoBootResource,
    AmbiguousBootResource,
    UnsupportedBootResource,
    Unexpected,
}

impl From<AnyhowError> for BootError {
    fn from(_: AnyhowError) -> Self {
        BootError::Unexpected
    }
}

impl From<BootError> for ErrorKind {
    fn from(error: BootError) -> Self {
        match error {
            BootError::NoBootResource => ErrorKind::NotFound,
            BootError::AmbiguousBootResource => ErrorKind::InvalidEntry,
            BootError::UnsupportedBootResource => ErrorKind::InvalidEntry,
            BootError::Unexpected => ErrorKind::Unexpected,
        }
    }
}
