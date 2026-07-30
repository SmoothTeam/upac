use upac_abi::error::ErrorKind;

use crate::database::DatabaseError;
use crate::deploy::SysrootError;
use crate::types::errors::{CommonError, common_error_from, database_error_from, lock_error_from, sysroot_error_from};
use crate::types::lock::LockError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListCommitError {
    Common(CommonError),
}

common_error_from!(ListCommitError);

database_error_from!(ListCommitError);

sysroot_error_from!(ListCommitError);

lock_error_from!(ListCommitError);

impl From<ListCommitError> for ErrorKind {
    fn from(error: ListCommitError) -> Self {
        match error {
            ListCommitError::Common(common_error) => common_error.into(),
        }
    }
}
