// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::ffi::FromBytesWithNulError;
use std::str::Utf8Error;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorDomain {
    Uninstall,
    Install,
    Rollback,
    Commit,
    Files,
    Update,
    ListPackages,
    ListCommit,
    ListPrefix,
    ListHistory,
    DiffFiles,
    DiffPackages,
    Diff,
    SearchMeta,
    SearchFiles,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Unexpected,
    OutOfMemory,
    NotFound,
    AlreadyExists,
    PermissionDenied,
    InvalidPath,
    NoSpaceLeft,
    Cancelled,
    ReadFailed,
    WriteFailed,
    NotInitialized,
    AbiMismatch,
    InvalidEntry,
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CError {
    pub domain: ErrorDomain,
    pub state: u32,
    pub error: ErrorKind,
}

pub trait CommandState: Copy {
    const DOMAIN: ErrorDomain;
    const VALIDATION: Self;

    fn as_u32(self) -> u32;
}
