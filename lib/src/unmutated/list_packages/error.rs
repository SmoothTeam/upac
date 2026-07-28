use upac_abi::error::ErrorKind;

use crate::database::DatabaseError;
use crate::deploy::SysrootError;
use crate::types::errors::CommonError;
use crate::types::lock::LockError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListPackagesError {
    Common(CommonError),
}

impl From<CommonError> for ListPackagesError {
    fn from(error: CommonError) -> Self {
        ListPackagesError::Common(error)
    }
}

impl From<DatabaseError> for ListPackagesError {
    fn from(error: DatabaseError) -> Self {
        ListPackagesError::Common(CommonError::Database(error))
    }
}

impl From<SysrootError> for ListPackagesError {
    fn from(error: SysrootError) -> Self {
        ListPackagesError::Common(CommonError::Sysroot(error))
    }
}

impl From<LockError> for ListPackagesError {
    fn from(error: LockError) -> Self {
        ListPackagesError::Common(CommonError::Lock(error))
    }
}

impl From<ListPackagesError> for ErrorKind {
    fn from(error: ListPackagesError) -> Self {
        match error {
            ListPackagesError::Common(common_error) => common_error.into(),
        }
    }
}
