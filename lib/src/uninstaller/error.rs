use upac_abi::error::ErrorKind;

use crate::database::DatabaseError;
use crate::deploy::SysrootError;
use crate::types::errors::CommonError;
use crate::types::lock::LockError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UninstallError {
    PackageNotFound,
    UninstallFailed,
    FileMapCorrupted,
    StagingNotCleaned,
    CheckoutFailed,
    ReadDatabaseFailed,
    WriteDatabaseFailed,
    Common(CommonError),
}

impl From<CommonError> for UninstallError {
    fn from(error: CommonError) -> Self {
        UninstallError::Common(error)
    }
}

impl From<DatabaseError> for UninstallError {
    fn from(error: DatabaseError) -> Self {
        UninstallError::Common(CommonError::Database(error))
    }
}

impl From<SysrootError> for UninstallError {
    fn from(error: SysrootError) -> Self {
        UninstallError::Common(CommonError::Sysroot(error))
    }
}

impl From<LockError> for UninstallError {
    fn from(error: LockError) -> Self {
        UninstallError::Common(CommonError::Lock(error))
    }
}

impl From<UninstallError> for ErrorKind {
    fn from(error: UninstallError) -> Self {
        match error {
            UninstallError::PackageNotFound => ErrorKind::NotFound,
            UninstallError::UninstallFailed => ErrorKind::Unexpected,
            UninstallError::FileMapCorrupted => ErrorKind::Unexpected,
            UninstallError::StagingNotCleaned => ErrorKind::Unexpected,
            UninstallError::CheckoutFailed => ErrorKind::WriteFailed,
            UninstallError::ReadDatabaseFailed => ErrorKind::ReadFailed,
            UninstallError::WriteDatabaseFailed => ErrorKind::WriteFailed,
            UninstallError::Common(common_error) => common_error.into(),
        }
    }
}
