use upac_abi::error::ErrorKind;

use crate::database::DatabaseError;
use crate::deploy::SysrootError;
use crate::types::errors::{CommonError, common_error_from, database_error_from, lock_error_from, sysroot_error_from};
use crate::types::lock::LockError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListPackagesError {
    Common(CommonError),
}

common_error_from!(ListPackagesError);

database_error_from!(ListPackagesError);

sysroot_error_from!(ListPackagesError);

lock_error_from!(ListPackagesError);

impl From<ListPackagesError> for ErrorKind {
    fn from(error: ListPackagesError) -> Self {
        match error {
            ListPackagesError::Common(common_error) => common_error.into(),
        }
    }
}
