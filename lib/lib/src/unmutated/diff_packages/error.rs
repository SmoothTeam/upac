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
pub enum DiffPackagesError {
    Common(CommonError),
}

common_error_from!(DiffPackagesError);

database_error_from!(DiffPackagesError);

sysroot_error_from!(DiffPackagesError);

lock_error_from!(DiffPackagesError);

impl From<DiffPackagesError> for ErrorKind {
    fn from(error: DiffPackagesError) -> Self {
        match error {
            DiffPackagesError::Common(common_error) => common_error.into(),
        }
    }
}
