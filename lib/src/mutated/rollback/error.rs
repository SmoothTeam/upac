use upac_abi::error::ErrorKind;

use crate::database::DatabaseError;
use crate::deploy::SysrootError;
use crate::types::errors::CommonError;
use crate::types::lock::LockError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollbackError {
    Common(CommonError),
}

impl From<CommonError> for RollbackError {
    fn from(error: CommonError) -> Self {
        RollbackError::Common(error)
    }
}

impl From<DatabaseError> for RollbackError {
    fn from(error: DatabaseError) -> Self {
        RollbackError::Common(CommonError::Database(error))
    }
}

impl From<SysrootError> for RollbackError {
    fn from(error: SysrootError) -> Self {
        RollbackError::Common(CommonError::Sysroot(error))
    }
}

impl From<LockError> for RollbackError {
    fn from(error: LockError) -> Self {
        RollbackError::Common(CommonError::Lock(error))
    }
}

impl From<RollbackError> for ErrorKind {
    fn from(error: RollbackError) -> Self {
        match error {
            RollbackError::Common(common_error) => common_error.into(),
        }
    }
}
