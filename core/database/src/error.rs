// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::Error as IoError;

use redb::CommitError as RedbCommitError;
use redb::DatabaseError as RedbDatabaseError;
use redb::StorageError as RedbStorageError;
use redb::TableError as RedbTableError;
use redb::TransactionError as RedbTransactionError;

use upac_abi::error::ErrorKind;

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

impl From<RedbStorageError> for DatabaseError {
    fn from(error: RedbStorageError) -> Self {
        match error {
            RedbStorageError::Corrupted(_) => DatabaseError::ReadError,
            RedbStorageError::DatabaseClosed => DatabaseError::DatabaseNotInitialized,
            RedbStorageError::LockPoisoned(_) => DatabaseError::WriteError,
            RedbStorageError::ValueTooLarge(_) | RedbStorageError::Io(_) | RedbStorageError::PreviousIo => {
                DatabaseError::WriteError
            }
            _ => DatabaseError::ReadError,
        }
    }
}

impl From<RedbTableError> for DatabaseError {
    fn from(error: RedbTableError) -> Self {
        match error {
            RedbTableError::TableDoesNotExist(_) => DatabaseError::DatabaseNotInitialized,
            RedbTableError::Storage(storage_error) => storage_error.into(),
            _ => DatabaseError::WriteError,
        }
    }
}

impl From<RedbTransactionError> for DatabaseError {
    fn from(error: RedbTransactionError) -> Self {
        match error {
            RedbTransactionError::Storage(storage_error) => storage_error.into(),
            _ => DatabaseError::WriteError,
        }
    }
}

impl From<RedbCommitError> for DatabaseError {
    fn from(error: RedbCommitError) -> Self {
        match error {
            RedbCommitError::Storage(storage_error) => storage_error.into(),
            _ => DatabaseError::WriteError,
        }
    }
}

impl From<RedbDatabaseError> for DatabaseError {
    fn from(error: RedbDatabaseError) -> Self {
        match error {
            RedbDatabaseError::DatabaseAlreadyOpen => DatabaseError::AccessDenied,
            RedbDatabaseError::Storage(storage_error) => storage_error.into(),
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
