use std::io::Error as IoError;

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
