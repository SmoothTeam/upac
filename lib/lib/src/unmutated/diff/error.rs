// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::error::ErrorKind;

use crate::database::DatabaseError;
use crate::deploy::SysrootError;
use crate::types::errors::{CommonError, common_error_from, database_error_from, lock_error_from, sysroot_error_from};
use crate::types::lock::LockError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffError {
    Common(CommonError),
}

common_error_from!(DiffError);

database_error_from!(DiffError);

sysroot_error_from!(DiffError);

lock_error_from!(DiffError);

impl From<DiffError> for ErrorKind {
    fn from(error: DiffError) -> Self {
        match error {
            DiffError::Common(common_error) => common_error.into(),
        }
    }
}
