use upac_abi::error::ErrorKind;

use crate::database::DatabaseError;
use crate::deploy::SysrootError;
use crate::types::errors::{CommonError, common_error_from, database_error_from, lock_error_from, sysroot_error_from};
use crate::types::lock::LockError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitError {
    Common(CommonError),
}

common_error_from!(CommitError);

database_error_from!(CommitError);

sysroot_error_from!(CommitError);

lock_error_from!(CommitError);

impl From<CommitError> for ErrorKind {
    fn from(error: CommitError) -> Self {
        match error {
            CommitError::Common(common_error) => common_error.into(),
        }
    }
}
