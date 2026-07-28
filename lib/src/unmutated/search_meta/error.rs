use upac_abi::error::ErrorKind;

use crate::database::DatabaseError;
use crate::deploy::SysrootError;
use crate::types::errors::CommonError;
use crate::types::lock::LockError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMetaError {
    Common(CommonError),
}

impl From<CommonError> for SearchMetaError {
    fn from(error: CommonError) -> Self {
        SearchMetaError::Common(error)
    }
}

impl From<DatabaseError> for SearchMetaError {
    fn from(error: DatabaseError) -> Self {
        SearchMetaError::Common(CommonError::Database(error))
    }
}

impl From<SysrootError> for SearchMetaError {
    fn from(error: SysrootError) -> Self {
        SearchMetaError::Common(CommonError::Sysroot(error))
    }
}

impl From<LockError> for SearchMetaError {
    fn from(error: LockError) -> Self {
        SearchMetaError::Common(CommonError::Lock(error))
    }
}

impl From<SearchMetaError> for ErrorKind {
    fn from(error: SearchMetaError) -> Self {
        match error {
            SearchMetaError::Common(common_error) => common_error.into(),
        }
    }
}
