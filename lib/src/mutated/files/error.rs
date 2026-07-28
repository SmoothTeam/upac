use upac_abi::error::ErrorKind;

use crate::database::DatabaseError;
use crate::deploy::SysrootError;
use crate::types::errors::CommonError;
use crate::types::lock::LockError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesError {
    Common(CommonError),
}

impl From<CommonError> for FilesError {
    fn from(error: CommonError) -> Self {
        FilesError::Common(error)
    }
}

impl From<DatabaseError> for FilesError {
    fn from(error: DatabaseError) -> Self {
        FilesError::Common(CommonError::Database(error))
    }
}

impl From<SysrootError> for FilesError {
    fn from(error: SysrootError) -> Self {
        FilesError::Common(CommonError::Sysroot(error))
    }
}

impl From<LockError> for FilesError {
    fn from(error: LockError) -> Self {
        FilesError::Common(CommonError::Lock(error))
    }
}

impl From<FilesError> for ErrorKind {
    fn from(error: FilesError) -> Self {
        match error {
            FilesError::Common(common_error) => common_error.into(),
        }
    }
}
