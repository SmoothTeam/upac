use upac_abi::error::ErrorKind;

use crate::database::DatabaseError;
use crate::deploy::SysrootError;
use crate::types::errors::{CommonError, common_error_from, database_error_from, lock_error_from, sysroot_error_from};
use crate::types::lock::LockError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesError {
    Common(CommonError),
}

common_error_from!(FilesError);

database_error_from!(FilesError);

sysroot_error_from!(FilesError);

lock_error_from!(FilesError);

impl From<FilesError> for ErrorKind {
    fn from(error: FilesError) -> Self {
        match error {
            FilesError::Common(common_error) => common_error.into(),
        }
    }
}
