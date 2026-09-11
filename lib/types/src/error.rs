// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;
use std::str::Utf8Error;

use upac_abi::error::{CError, ErrorKind};

use super::traits::CommandState;

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

pub unsafe fn write_error<S: CommandState>(err_out: *mut CError, state: S, error: ErrorKind) {
    if !err_out.is_null() {
        unsafe {
            *err_out = CError {
                domain: S::DOMAIN,
                state: state.as_u32(),
                error,
            };
        }
    }
}

pub fn write_abi_error<S: CommandState>(error: ErrorKind, err_out: *mut CError) -> i32 {
    unsafe { write_error(err_out, S::VALIDATION, error) };
    -1
}

#[macro_export]
macro_rules! try_convert_abi {
    ($expr:expr, $err_out:expr, $state:ty) => {
        match $expr {
            Ok(value) => value,
            Err(error) => return upac_types::error::write_abi_error::<$state>(error, $err_out),
        }
    };
}
pub use try_convert_abi;
