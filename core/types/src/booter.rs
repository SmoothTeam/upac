// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::CEnum;

use crate::error::ErrorKind;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, CEnum)]
pub enum BootResourceKind {
    Bls = 0,
    Uki = 1,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, CEnum)]
pub enum BootError {
    InvalidRequest = 1,
    PermissionDenied = 2,
    EntryNotFound = 3,
    ToolNotInstalled = 4,
    EfiUnavailable = 5,
    NoFreeBootId = 6,
    Unexpected = 7,
}

impl From<BootError> for ErrorKind {
    fn from(error: BootError) -> Self {
        match error {
            BootError::InvalidRequest => ErrorKind::InvalidEntry,
            BootError::PermissionDenied => ErrorKind::PermissionDenied,
            BootError::EntryNotFound => ErrorKind::NotFound,
            BootError::ToolNotInstalled => ErrorKind::ToolNotInstalled,
            BootError::EfiUnavailable => ErrorKind::NotInitialized,
            BootError::NoFreeBootId => ErrorKind::NoSpaceLeft,
            BootError::Unexpected => ErrorKind::Unexpected,
        }
    }
}
