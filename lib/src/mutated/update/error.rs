use upac_abi::error::ErrorKind;

use crate::database::DatabaseError;
use crate::deploy::SysrootError;
use crate::types::errors::CommonError;
use crate::types::lock::LockError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateError {
    Common(CommonError),
}

impl From<CommonError> for UpdateError {
    fn from(error: CommonError) -> Self {
        UpdateError::Common(error)
    }
}

impl From<DatabaseError> for UpdateError {
    fn from(error: DatabaseError) -> Self {
        UpdateError::Common(CommonError::Database(error))
    }
}

impl From<SysrootError> for UpdateError {
    fn from(error: SysrootError) -> Self {
        UpdateError::Common(CommonError::Sysroot(error))
    }
}

impl From<LockError> for UpdateError {
    fn from(error: LockError) -> Self {
        UpdateError::Common(CommonError::Lock(error))
    }
}

impl From<UpdateError> for ErrorKind {
    fn from(error: UpdateError) -> Self {
        match error {
            UpdateError::Common(common_error) => common_error.into(),
        }
    }
}
