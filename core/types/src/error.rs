// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::FromBytesWithNulError;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::str::Utf8Error;

use upac_abi::error::{AbiError, CError};

use upac_macro::{CEnum, CTryToRust, RustToC};

use super::traits::CommandState;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, CEnum)]
pub enum ErrorDomain {
    Unknown,
    Uninstall,
    Install,
    Rollback,
    Commit,
    Files,
    Update,
    Gc,
    Pin,
    Mime,
    ListPackages,
    ListConfig,
    ListPrefix,
    ListHistory,
    DiffPrefix,
    DiffConfig,
    DiffPackages,
    Diff,
    SearchMeta,
    SearchFiles,
    SearchInMeta,
    SearchInPackageFiles,
    Bootstrap,
    PartitionTable,
    PartitionAdd,
    Format,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, CEnum)]
pub enum ErrorKind {
    Unexpected = 1,
    OutOfMemory = 2,
    NotFound = 3,
    AlreadyExists = 4,
    PermissionDenied = 5,
    InvalidPath = 6,
    NoSpaceLeft = 7,
    Cancelled = 8,
    ReadFailed = 9,
    WriteFailed = 10,
    NotInitialized = 11,
    AbiMismatch = 12,
    InvalidEntry = 13,
    NotAPartition = 14,
    WrongPartitionType = 15,
    UnsupportedFilesystem = 16,
    ToolNotInstalled = 17,
    ToolFailed = 18,
    RollbackFailed = 19,
}

impl From<AbiError> for ErrorKind {
    fn from(error: AbiError) -> Self {
        match error {
            AbiError::InvalidEntry => ErrorKind::InvalidEntry,
            AbiError::AbiMismatch => ErrorKind::AbiMismatch,
        }
    }
}

impl From<FromBytesWithNulError> for ErrorKind {
    fn from(_: FromBytesWithNulError) -> Self {
        ErrorKind::InvalidEntry
    }
}

impl From<Utf8Error> for ErrorKind {
    fn from(_: Utf8Error) -> Self {
        ErrorKind::InvalidEntry
    }
}

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

        let converted = unsafe { error.validate() }
            .map_err(ErrorKind::from)
            .and_then(|()| Error::try_from(error));

        Err(converted.unwrap_or_else(|kind| Error {
            domain: ErrorDomain::try_from(error.domain).unwrap_or(ErrorDomain::Unknown),
            state: error.state,
            kind,
        }))
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
