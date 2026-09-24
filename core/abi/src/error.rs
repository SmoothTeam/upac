// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::FromBytesWithNulError;
use std::mem::size_of;
use std::str::Utf8Error;

use upac_macro::CValidate;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorDomain {
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
#[derive(Debug, Clone, Copy, CValidate)]
pub struct CError {
    pub struct_size: usize,

    pub domain: ErrorDomain,
    pub state: u32,
    pub kind: ErrorKind,
}

impl Default for CError {
    fn default() -> Self {
        CError {
            struct_size: size_of::<CError>(),
            domain: ErrorDomain::Uninstall,
            state: 0,
            kind: ErrorKind::Unexpected,
        }
    }
}
