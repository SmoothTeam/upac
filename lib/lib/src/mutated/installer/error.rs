use upac_abi::error::ErrorKind;

use crate::database::DatabaseError;
use crate::deploy::SysrootError;
use crate::types::errors::{CommonError, common_error_from, database_error_from, lock_error_from, sysroot_error_from};
use crate::types::lock::LockError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallError {
    Common(CommonError),
}

common_error_from!(InstallError);

database_error_from!(InstallError);

sysroot_error_from!(InstallError);

lock_error_from!(InstallError);

impl From<InstallError> for ErrorKind {
    fn from(error: InstallError) -> Self {
        match error {
            InstallError::Common(common_error) => common_error.into(),
        }
    }
}
