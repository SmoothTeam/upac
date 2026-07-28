use upac_abi::error::ErrorKind;

use crate::database::DatabaseError;
use crate::deploy::SysrootError;
use crate::types::errors::CommonError;
use crate::types::lock::LockError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchFilesError {
    Common(CommonError),
}

impl From<CommonError> for SearchFilesError {
    fn from(error: CommonError) -> Self {
        SearchFilesError::Common(error)
    }
}

impl From<DatabaseError> for SearchFilesError {
    fn from(error: DatabaseError) -> Self {
        SearchFilesError::Common(CommonError::Database(error))
    }
}

impl From<SysrootError> for SearchFilesError {
    fn from(error: SysrootError) -> Self {
        SearchFilesError::Common(CommonError::Sysroot(error))
    }
}

impl From<LockError> for SearchFilesError {
    fn from(error: LockError) -> Self {
        SearchFilesError::Common(CommonError::Lock(error))
    }
}

impl From<SearchFilesError> for ErrorKind {
    fn from(error: SearchFilesError) -> Self {
        match error {
            SearchFilesError::Common(common_error) => common_error.into(),
        }
    }
}
