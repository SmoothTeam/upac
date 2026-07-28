use upac_abi::error::ErrorKind;

use crate::database::DatabaseError;
use crate::deploy::SysrootError;
use crate::types::errors::CommonError;
use crate::types::lock::LockError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitError {
    Common(CommonError),
}

impl From<CommonError> for CommitError {
    fn from(error: CommonError) -> Self {
        CommitError::Common(error)
    }
}

impl From<DatabaseError> for CommitError {
    fn from(error: DatabaseError) -> Self {
        CommitError::Common(CommonError::Database(error))
    }
}

impl From<SysrootError> for CommitError {
    fn from(error: SysrootError) -> Self {
        CommitError::Common(CommonError::Sysroot(error))
    }
}

impl From<LockError> for CommitError {
    fn from(error: LockError) -> Self {
        CommitError::Common(CommonError::Lock(error))
    }
}

impl From<CommitError> for ErrorKind {
    fn from(error: CommitError) -> Self {
        match error {
            CommitError::Common(common_error) => common_error.into(),
        }
    }
}
