// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

use upac_abi::error::{CError, ErrorKind};

#[derive(Debug)]
pub struct AbiMismatch {
    pub got: u32,
    pub expected: u32,
}

impl Display for AbiMismatch {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(
            formatter,
            "{} ({} → {})",
            gettextrs::gettext("abi_version_mismatch"),
            self.got,
            self.expected
        )
    }
}

impl Error for AbiMismatch {}

#[derive(Debug)]
pub struct LibError {
    pub error: CError,
}

impl Display for LibError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let key = match self.error.error {
            ErrorKind::Unexpected => "err_unexpected",
            ErrorKind::OutOfMemory => "err_oom",
            ErrorKind::NotFound => "err_not_found",
            ErrorKind::AlreadyExists => "err_already_exists",
            ErrorKind::PermissionDenied => "err_permission_denied",
            ErrorKind::InvalidPath => "err_invalid_path",
            ErrorKind::NoSpaceLeft => "err_no_space",
            ErrorKind::Cancelled => "err_cancelled",
            ErrorKind::ReadFailed => "err_read",
            ErrorKind::WriteFailed => "err_write",
            ErrorKind::NotInitialized => "err_not_initialized",
            ErrorKind::AbiMismatch => "err_abi_mismatch",
            ErrorKind::InvalidEntry => "err_invalid_entry",
        };
        write!(
            formatter,
            "{} ({:?}, state {})",
            gettextrs::gettext(key),
            self.error.domain,
            self.error.state
        )
    }
}

impl Error for LibError {}
