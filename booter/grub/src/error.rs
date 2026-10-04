// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use upac_types::booter::BootError;
use upac_types::error::ErrorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrubError {
    ToolNotFound,
    PermissionDenied,
    InvalidRequest,
    Unexpected,
}

impl From<IoError> for GrubError {
    fn from(error: IoError) -> Self {
        match error.kind() {
            IoErrorKind::NotFound => GrubError::ToolNotFound,
            IoErrorKind::PermissionDenied => GrubError::PermissionDenied,
            _ => GrubError::Unexpected,
        }
    }
}

impl From<ErrorKind> for GrubError {
    fn from(error: ErrorKind) -> Self {
        match error {
            ErrorKind::PermissionDenied => GrubError::PermissionDenied,
            _ => GrubError::InvalidRequest,
        }
    }
}

impl From<GrubError> for BootError {
    fn from(error: GrubError) -> Self {
        match error {
            GrubError::ToolNotFound => BootError::ToolNotInstalled,
            GrubError::PermissionDenied => BootError::PermissionDenied,
            GrubError::InvalidRequest => BootError::InvalidRequest,
            GrubError::Unexpected => BootError::Unexpected,
        }
    }
}
