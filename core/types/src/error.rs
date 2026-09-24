// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::str::Utf8Error;

use upac_abi::error::{CError, ErrorDomain, ErrorKind};

use upac_macro::{CTryToRust, RustToC};

use super::traits::CommandState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, CTryToRust, RustToC)]
pub struct Error {
    pub domain: ErrorDomain,
    pub state: u32,
    pub kind: ErrorKind,
}

impl Error {
    pub fn new<S: CommandState>(state: S, kind: ErrorKind) -> Self {
        Error {
            domain: S::DOMAIN,
            state: state.as_u32(),
            kind,
        }
    }

    pub fn catch<S: CommandState, T, E: Into<ErrorKind>>(call: impl FnOnce() -> Result<T, (S, E)>) -> Result<T, Self> {
        match catch_unwind(AssertUnwindSafe(call)) {
            Ok(Ok(value)) => Ok(value),
            Ok(Err((state, error))) => Err(Error::new(state, error.into())),
            Err(_) => Err(Error::new(S::VALIDATION, ErrorKind::Unexpected)),
        }
    }

    pub fn check(code: i32, error: &CError) -> Result<(), Self> {
        if code == 0 {
            return Ok(());
        }

        let converted = unsafe { error.validate() }.and_then(|()| Error::try_from(error));

        Err(converted.unwrap_or_else(|kind| Error {
            domain: error.domain,
            state: error.state,
            kind,
        }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    InvalidRequest,
    Io(IoErrorKind),
    ChecksumMismatch,
    UnsupportedFormat,
    MissingMetadata,
    MalformedMetadata,
    InvalidUtf8,
    Cancelled,
}

impl From<IoError> for DecodeError {
    fn from(error: IoError) -> Self {
        DecodeError::Io(error.kind())
    }
}

impl From<Utf8Error> for DecodeError {
    fn from(_: Utf8Error) -> Self {
        DecodeError::InvalidRequest
    }
}

impl DecodeError {
    pub fn code(self) -> i32 {
        match self {
            DecodeError::InvalidRequest => -1,
            DecodeError::Io(_) => -2,
            DecodeError::ChecksumMismatch => -3,
            DecodeError::UnsupportedFormat => -4,
            DecodeError::MissingMetadata => -5,
            DecodeError::MalformedMetadata => -6,
            DecodeError::InvalidUtf8 => -7,
            DecodeError::Cancelled => -8,
        }
    }
}

/// # Safety
/// `err_out`, if non-null, must point to writable `CError` storage.
pub unsafe fn write_abi_error(err_out: *mut CError, error: Error) -> i32 {
    if !err_out.is_null() {
        unsafe { *err_out = error.into() };
    }

    -1
}

#[macro_export]
macro_rules! try_convert_abi {
    ($expr:expr, $err_out:expr, $state:ty) => {
        match $expr {
            Ok(value) => value,
            Err(error) => {
                return unsafe {
                    upac_types::error::write_abi_error(
                        $err_out,
                        upac_types::error::Error::new(<$state as upac_types::traits::CommandState>::VALIDATION, error),
                    )
                };
            }
        }
    };
}
pub use try_convert_abi;
