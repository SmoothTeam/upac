// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use upac_abi::error::ErrorKind;

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

impl From<GrubError> for ErrorKind {
    fn from(error: GrubError) -> Self {
        match error {
            GrubError::ToolNotFound => ErrorKind::NotFound,
            GrubError::PermissionDenied => ErrorKind::PermissionDenied,
            GrubError::InvalidRequest => ErrorKind::InvalidEntry,
            GrubError::Unexpected => ErrorKind::Unexpected,
        }
    }
}
