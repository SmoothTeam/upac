use upac_abi::error::ErrorKind;

use crate::database::DatabaseError;
use crate::deploy::SysrootError;
use crate::types::errors::{CommonError, common_error_from, database_error_from, lock_error_from, sysroot_error_from};
use crate::types::lock::LockError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListPrefixError {
    Common(CommonError),
}

common_error_from!(ListPrefixError);

database_error_from!(ListPrefixError);

sysroot_error_from!(ListPrefixError);

lock_error_from!(ListPrefixError);

impl From<ListPrefixError> for ErrorKind {
    fn from(error: ListPrefixError) -> Self {
        match error {
            ListPrefixError::Common(common_error) => common_error.into(),
        }
    }
}
