// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::error::ErrorKind;

use crate::database::error::DatabaseError;
use crate::deploy::error::SysrootError;
use crate::errors::{CommonError, common_error_from, database_error_from, lock_error_from, sysroot_error_from};
use crate::lock::LockError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollbackError {
    Common(CommonError),
}

common_error_from!(RollbackError);

database_error_from!(RollbackError);

sysroot_error_from!(RollbackError);

lock_error_from!(RollbackError);

impl From<RollbackError> for ErrorKind {
    fn from(error: RollbackError) -> Self {
        match error {
            RollbackError::Common(common_error) => common_error.into(),
        }
    }
}
