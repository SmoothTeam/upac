// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::error::ErrorKind;

use crate::boot::error::BootError;
use crate::database::error::DatabaseError;
use crate::deploy::error::SysrootError;
use crate::errors::{
    CommonError, boot_error_from, common_error_from, database_error_from, lock_error_from, sysroot_error_from,
};
use crate::lock::LockError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UninstallError {
    PackageNotFound,
    FileMapCorrupted,
    StagingNotCleaned,
    CheckoutFailed,
    ReadDatabaseFailed,
    WriteDatabaseFailed,
    Common(CommonError),
}

common_error_from!(UninstallError);

database_error_from!(UninstallError);

sysroot_error_from!(UninstallError);

lock_error_from!(UninstallError);

boot_error_from!(UninstallError);

impl From<UninstallError> for ErrorKind {
    fn from(error: UninstallError) -> Self {
        match error {
            UninstallError::PackageNotFound => ErrorKind::NotFound,
            UninstallError::FileMapCorrupted => ErrorKind::Unexpected,
            UninstallError::StagingNotCleaned => ErrorKind::Unexpected,
            UninstallError::CheckoutFailed => ErrorKind::WriteFailed,
            UninstallError::ReadDatabaseFailed => ErrorKind::ReadFailed,
            UninstallError::WriteDatabaseFailed => ErrorKind::WriteFailed,
            UninstallError::Common(common_error) => common_error.into(),
        }
    }
}
