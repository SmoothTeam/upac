// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::FromBytesWithNulError;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
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
    Attach,
    Detach,
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

impl From<IoError> for ErrorKind {
    fn from(error: IoError) -> Self {
        match error.kind() {
            IoErrorKind::NotFound => ErrorKind::NotFound,
            IoErrorKind::PermissionDenied => ErrorKind::PermissionDenied,
            IoErrorKind::AlreadyExists => ErrorKind::AlreadyExists,
            IoErrorKind::StorageFull => ErrorKind::NoSpaceLeft,
            _ => ErrorKind::Unexpected,
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
pub unsafe fn export_mutated_command<'request, CRequest, Request, State>(
    request: &'request CRequest, err_out: *mut CError, run: impl FnOnce(Request) -> Result<(), (State, ErrorKind)>,
) -> i32
where
    Request: TryFrom<&'request CRequest, Error = ErrorKind>,
    State: CommandState,
{
    match call_exported(request, run) {
        Ok(()) => 0,
        Err(error) => unsafe { write_error(err_out, error) },
    }
}

/// # Safety
/// `response_out`, if non-null, must point to writable `CResponse` storage, and `err_out`, if non-null, to
/// writable `CError` storage.
pub unsafe fn export_unmutated_command<'request, CRequest, Request, State, Response, CResponse>(
    request: &'request CRequest, response_out: *mut CResponse, err_out: *mut CError,
    run: impl FnOnce(Request) -> Result<Response, (State, ErrorKind)>,
) -> i32
where
    Request: TryFrom<&'request CRequest, Error = ErrorKind>,
    State: CommandState,
    CResponse: From<Response>,
{
    match call_exported(request, run) {
        Ok(response) => {
            if !response_out.is_null() {
                unsafe { response_out.write(CResponse::from(response)) };
            }
            0
        }
        Err(error) => unsafe { write_error(err_out, error) },
    }
}

fn call_exported<'request, CRequest, Request, State, Response>(
    request: &'request CRequest, run: impl FnOnce(Request) -> Result<Response, (State, ErrorKind)>,
) -> Result<Response, Error>
where
    Request: TryFrom<&'request CRequest, Error = ErrorKind>,
    State: CommandState,
{
    let request = Request::try_from(request).map_err(|kind| Error::new(State::VALIDATION, kind))?;

    match catch_unwind(AssertUnwindSafe(|| run(request))) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err((state, kind))) => Err(Error::new(state, kind)),
        Err(_) => Err(Error::new(State::VALIDATION, ErrorKind::Unexpected)),
    }
}

unsafe fn write_error(err_out: *mut CError, error: Error) -> i32 {
    if !err_out.is_null() {
        unsafe { err_out.write(error.into()) };
    }

    -1
}
