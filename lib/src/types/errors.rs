use upac_abi::error::ErrorKind;

use crate::composefs::RepoError;
use crate::database::DatabaseError;
use crate::deploy::SysrootError;
use crate::types::lock::LockError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonError {
    OutOfMemory,
    Cancelled,
    AccessDenied,
    MaxRetriesExceeded,
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
            CommonError::MaxRetriesExceeded => ErrorKind::MaxRetriesExceeded,
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
