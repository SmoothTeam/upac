// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

use i18n_embed_fl::fl;

use upac_abi::error::{CError, ErrorKind};

use upac_types::states::SetupStateId;

use crate::locale::LOADER;

#[cfg(test)]
#[path = "../../tests/inline/errors.rs"]
mod tests;

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
            fl!(LOADER, "abi-version-mismatch"),
            self.got,
            self.expected
        )
    }
}

impl Error for AbiMismatch {}

fn error_kind_message(kind: ErrorKind) -> String {
    match kind {
        ErrorKind::Unexpected => fl!(LOADER, "err-unexpected"),
        ErrorKind::OutOfMemory => fl!(LOADER, "err-oom"),
        ErrorKind::NotFound => fl!(LOADER, "err-not-found"),
        ErrorKind::AlreadyExists => fl!(LOADER, "err-already-exists"),
        ErrorKind::PermissionDenied => fl!(LOADER, "err-permission-denied"),
        ErrorKind::InvalidPath => fl!(LOADER, "err-invalid-path"),
        ErrorKind::NoSpaceLeft => fl!(LOADER, "err-no-space"),
        ErrorKind::Cancelled => fl!(LOADER, "err-cancelled"),
        ErrorKind::ReadFailed => fl!(LOADER, "err-read"),
        ErrorKind::WriteFailed => fl!(LOADER, "err-write"),
        ErrorKind::NotInitialized => fl!(LOADER, "err-not-initialized"),
        ErrorKind::AbiMismatch => fl!(LOADER, "err-abi-mismatch"),
        ErrorKind::InvalidEntry => fl!(LOADER, "err-invalid-entry"),
    }
}

#[derive(Debug)]
pub struct LibError {
    pub error: CError,
}

impl LibError {
    /// # Safety
    /// `error` must point to a valid, initialized `CError` whenever `code != 0` — the ABI only writes
    /// to it on the failure path, leaving it uninitialized on success.
    pub unsafe fn check(code: i32, error: *const CError) -> Result<(), Self> {
        if code == 0 {
            return Ok(());
        }
        Err(Self {
            error: unsafe { *error },
        })
    }
}

impl Display for LibError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let stage = SetupStateId::from_stage_index(self.error.state as usize).stage_key();

        write!(
            formatter,
            "{}: {}",
            LOADER.get(stage),
            error_kind_message(self.error.error)
        )
    }
}

impl Error for LibError {}
