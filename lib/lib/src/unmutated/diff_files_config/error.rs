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
pub enum DiffFilesConfigError {
    Common(CommonError),
}

common_error_from!(DiffFilesConfigError);

database_error_from!(DiffFilesConfigError);

sysroot_error_from!(DiffFilesConfigError);

lock_error_from!(DiffFilesConfigError);

impl From<DiffFilesConfigError> for ErrorKind {
    fn from(error: DiffFilesConfigError) -> Self {
        match error {
            DiffFilesConfigError::Common(common_error) => common_error.into(),
        }
    }
}
