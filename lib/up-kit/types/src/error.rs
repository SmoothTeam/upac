// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::FromBytesWithNulError;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::str::Utf8Error;

#[cfg(feature = "gpt")]
use gptman::Error as GptError;
#[cfg(feature = "gpt")]
use gptman::linux::BlockError as GptBlockError;

use upac_abi::error::{AbiError, CError};

use upac_macro::{CEnum, CTryToRust, RustToC};

use crate::state::CommandState;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, CEnum)]
pub enum ErrorDomain {
    Unknown,
    Uninstall,
    Install,
    Rollback,
    Attach,
    Detach,
    Update,
    Gc,
    Pin,
    Mime,
    ListPackages,
    ListConfig,
    ListDeploys,
    ListHistory,
    Diff,
    SearchMeta,
    SearchFiles,
    SearchInMeta,
    SearchInPackageFiles,
    BootstrapImport,
    BootstrapKernel,
    BootstrapDeploy,
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

#[cfg(feature = "gpt")]
impl From<GptError> for ErrorKind {
    fn from(error: GptError) -> Self {
        match error {
            GptError::Io(io_error) => ErrorKind::from(io_error),
            GptError::InvalidSignature | GptError::ReadError(_, _) => ErrorKind::NotFound,
            GptError::NoSpaceLeft => ErrorKind::NoSpaceLeft,
            GptError::InvalidPartitionBoundaries => ErrorKind::InvalidEntry,
            _ => ErrorKind::Unexpected,
        }
    }
}

#[cfg(feature = "gpt")]
impl From<GptBlockError> for ErrorKind {
    fn from(error: GptBlockError) -> Self {
        match error {
            GptBlockError::Metadata(io_error) => ErrorKind::from(io_error),
            GptBlockError::NotBlock => ErrorKind::InvalidEntry,
            GptBlockError::RereadTable(_) => ErrorKind::ReadFailed,
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

#[derive(Debug, Clone, PartialEq, Eq, CTryToRust, RustToC)]
pub struct Error {
    pub domain: ErrorDomain,
    pub state: u32,
    pub kind: ErrorKind,
    pub subject: Option<String>,
}

impl Error {
    pub fn new<State: CommandState>(state: State, kind: ErrorKind) -> Self {
        Error {
            domain: State::DOMAIN,
            state: state.as_u32(),
            kind,
            subject: None,
        }
    }

    pub fn with_subject(mut self, subject: Option<String>) -> Self {
        self.subject = subject;
        self
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
            subject: None,
        }))
    }
}
