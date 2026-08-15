// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::io::Error as IoError;

use upac_abi::error::ErrorKind;

use crate::deploy::error::SysrootError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseError {
    PackageNotFound,
    FileNotFound,
    PackageAlreadyExists,
    PackageFilesExist,
    ArchitectureNotFound,
    DatabaseNotInitialized,
    AllocZFailed,
    AccessDenied,
    WriteError,
    ReadError,
}

impl From<redb::StorageError> for DatabaseError {
    fn from(error: redb::StorageError) -> Self {
        match error {
            redb::StorageError::Corrupted(_) => DatabaseError::ReadError,
            redb::StorageError::DatabaseClosed => DatabaseError::DatabaseNotInitialized,
            redb::StorageError::LockPoisoned(_) => DatabaseError::WriteError,
            redb::StorageError::ValueTooLarge(_) | redb::StorageError::Io(_) | redb::StorageError::PreviousIo => {
                DatabaseError::WriteError
            }
            _ => DatabaseError::ReadError,
        }
    }
}

impl From<redb::TableError> for DatabaseError {
    fn from(error: redb::TableError) -> Self {
        match error {
            redb::TableError::TableDoesNotExist(_) => DatabaseError::DatabaseNotInitialized,
            redb::TableError::Storage(storage_error) => storage_error.into(),
            _ => DatabaseError::WriteError,
        }
    }
}

impl From<redb::TransactionError> for DatabaseError {
    fn from(error: redb::TransactionError) -> Self {
        match error {
            redb::TransactionError::Storage(storage_error) => storage_error.into(),
            _ => DatabaseError::WriteError,
        }
    }
}

impl From<redb::CommitError> for DatabaseError {
    fn from(error: redb::CommitError) -> Self {
        match error {
            redb::CommitError::Storage(storage_error) => storage_error.into(),
            _ => DatabaseError::WriteError,
        }
    }
}

impl From<redb::DatabaseError> for DatabaseError {
    fn from(error: redb::DatabaseError) -> Self {
        match error {
            redb::DatabaseError::DatabaseAlreadyOpen => DatabaseError::AccessDenied,
            redb::DatabaseError::Storage(storage_error) => storage_error.into(),
            _ => DatabaseError::ReadError,
        }
    }
}

impl From<IoError> for DatabaseError {
    fn from(error: IoError) -> Self {
        match error.kind() {
            std::io::ErrorKind::PermissionDenied => DatabaseError::AccessDenied,
            _ => DatabaseError::WriteError,
        }
    }
}

impl From<DatabaseError> for ErrorKind {
    fn from(error: DatabaseError) -> Self {
        match error {
            DatabaseError::PackageNotFound => ErrorKind::NotFound,
            DatabaseError::FileNotFound => ErrorKind::NotFound,
            DatabaseError::PackageAlreadyExists => ErrorKind::AlreadyExists,
            DatabaseError::PackageFilesExist => ErrorKind::AlreadyExists,
            DatabaseError::ArchitectureNotFound => ErrorKind::NotFound,
            DatabaseError::DatabaseNotInitialized => ErrorKind::NotInitialized,
            DatabaseError::AllocZFailed => ErrorKind::OutOfMemory,
            DatabaseError::AccessDenied => ErrorKind::PermissionDenied,
            DatabaseError::WriteError => ErrorKind::WriteFailed,
            DatabaseError::ReadError => ErrorKind::ReadFailed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeployRecordError {
    NotFound,
    AccessDenied,
    MalformedJson,
    InvalidField,
    WriteFailed,
}

impl From<IoError> for DeployRecordError {
    fn from(error: IoError) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => DeployRecordError::NotFound,
            std::io::ErrorKind::PermissionDenied => DeployRecordError::AccessDenied,
            _ => DeployRecordError::WriteFailed,
        }
    }
}

impl From<serde_json::Error> for DeployRecordError {
    fn from(_: serde_json::Error) -> Self {
        DeployRecordError::MalformedJson
    }
}

impl From<DeployRecordError> for ErrorKind {
    fn from(error: DeployRecordError) -> Self {
        match error {
            DeployRecordError::NotFound => ErrorKind::NotFound,
            DeployRecordError::AccessDenied => ErrorKind::PermissionDenied,
            DeployRecordError::MalformedJson | DeployRecordError::InvalidField => ErrorKind::ReadFailed,
            DeployRecordError::WriteFailed => ErrorKind::WriteFailed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeployRecordsError {
    Sysroot(SysrootError),
    DeployRecord(DeployRecordError),
}

impl From<SysrootError> for DeployRecordsError {
    fn from(error: SysrootError) -> Self {
        DeployRecordsError::Sysroot(error)
    }
}

impl From<DeployRecordError> for DeployRecordsError {
    fn from(error: DeployRecordError) -> Self {
        DeployRecordsError::DeployRecord(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigDigestResolveError {
    Records(DeployRecordsError),
    NotFound(String),
}

impl From<DeployRecordsError> for ConfigDigestResolveError {
    fn from(error: DeployRecordsError) -> Self {
        ConfigDigestResolveError::Records(error)
    }
}
