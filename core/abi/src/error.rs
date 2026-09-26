// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::FromBytesWithNulError;
use std::mem::size_of;
use std::str::Utf8Error;

use upac_macro::{CEnum, CValidate};

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

    pub domain: u32,
    pub state: u32,
    pub kind: u32,
}

impl Default for CError {
    fn default() -> Self {
        CError {
            struct_size: size_of::<CError>(),
            domain: ErrorDomain::Unknown.into(),
            state: 0,
            kind: ErrorKind::Unexpected.into(),
        }
    }
}
