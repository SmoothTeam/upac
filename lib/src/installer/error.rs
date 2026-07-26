use upac_abi::error::ErrorKind;

use crate::database::DatabaseError;
use crate::deploy::SysrootError;
use crate::types::errors::CommonError;
use crate::types::lock::LockError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallError {
    Common(CommonError),
}

impl From<CommonError> for InstallError {
    fn from(error: CommonError) -> Self {
        InstallError::Common(error)
    }
}

impl From<DatabaseError> for InstallError {
    fn from(error: DatabaseError) -> Self {
        InstallError::Common(CommonError::Database(error))
    }
}

impl From<SysrootError> for InstallError {
    fn from(error: SysrootError) -> Self {
        InstallError::Common(CommonError::Sysroot(error))
    }
}

impl From<LockError> for InstallError {
    fn from(error: LockError) -> Self {
        InstallError::Common(CommonError::Lock(error))
    }
}

impl From<InstallError> for ErrorKind {
    fn from(error: InstallError) -> Self {
        match error {
            InstallError::Common(common_error) => common_error.into(),
        }
    }
}
