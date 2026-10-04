// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use anyhow::Error as AnyhowError;

use upac_types::error::ErrorKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootEntryError {
    NoBootResource,
    AmbiguousBootResource,
    UnsupportedBootResource,
    Unexpected,
}

impl From<AnyhowError> for BootEntryError {
    fn from(_: AnyhowError) -> Self {
        BootEntryError::Unexpected
    }
}

impl From<BootEntryError> for ErrorKind {
    fn from(error: BootEntryError) -> Self {
        match error {
            BootEntryError::NoBootResource => ErrorKind::NotFound,
            BootEntryError::AmbiguousBootResource => ErrorKind::InvalidEntry,
            BootEntryError::UnsupportedBootResource => ErrorKind::InvalidEntry,
            BootEntryError::Unexpected => ErrorKind::Unexpected,
        }
    }
}
