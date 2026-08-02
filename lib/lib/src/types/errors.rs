use upac_abi::error::ErrorKind;

use crate::composefs::RepoError;
use crate::database::DatabaseError;
use crate::deploy::SysrootError;
use crate::types::lock::LockError;

macro_rules! common_error_from {
    ($name:ident) => {
        impl From<CommonError> for $name {
            fn from(error: CommonError) -> Self {
                $name::Common(error)
            }
        }
    };
}
pub(crate) use common_error_from;

macro_rules! database_error_from {
    ($name:ident) => {
        impl From<DatabaseError> for $name {
            fn from(error: DatabaseError) -> Self {
                $name::Common(CommonError::Database(error))
            }
        }
    };
}
pub(crate) use database_error_from;

macro_rules! sysroot_error_from {
    ($name:ident) => {
        impl From<SysrootError> for $name {
            fn from(error: SysrootError) -> Self {
                $name::Common(CommonError::Sysroot(error))
            }
        }
    };
}
pub(crate) use sysroot_error_from;

macro_rules! lock_error_from {
    ($name:ident) => {
        impl From<LockError> for $name {
            fn from(error: LockError) -> Self {
                $name::Common(CommonError::Lock(error))
            }
        }
    };
}
pub(crate) use lock_error_from;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonError {
    OutOfMemory,
    Cancelled,
    AccessDenied,
    StageNotFound,
    MissingResult,
    Repo(RepoError),
    Database(DatabaseError),
    Sysroot(SysrootError),
    Lock(LockError),
}

impl From<CommonError> for ErrorKind {
    fn from(error: CommonError) -> Self {
        match error {
            CommonError::OutOfMemory => ErrorKind::OutOfMemory,
            CommonError::Cancelled => ErrorKind::Cancelled,
            CommonError::AccessDenied => ErrorKind::PermissionDenied,
            CommonError::StageNotFound => ErrorKind::Unexpected,
            CommonError::MissingResult => ErrorKind::Unexpected,
            CommonError::Repo(repo_error) => repo_error.into(),
            CommonError::Database(database_error) => database_error.into(),
            CommonError::Sysroot(sysroot_error) => sysroot_error.into(),
            CommonError::Lock(lock_error) => lock_error.into(),
        }
    }
}

impl From<RepoError> for CommonError {
    fn from(error: RepoError) -> Self {
        CommonError::Repo(error)
    }
}

impl From<DatabaseError> for CommonError {
    fn from(error: DatabaseError) -> Self {
        CommonError::Database(error)
    }
}

impl From<SysrootError> for CommonError {
    fn from(error: SysrootError) -> Self {
        CommonError::Sysroot(error)
    }
}

impl From<LockError> for CommonError {
    fn from(error: LockError) -> Self {
        CommonError::Lock(error)
    }
}
