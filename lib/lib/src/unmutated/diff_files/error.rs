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
pub enum DiffFilesError {
    Common(CommonError),
}

common_error_from!(DiffFilesError);

database_error_from!(DiffFilesError);

sysroot_error_from!(DiffFilesError);

lock_error_from!(DiffFilesError);

impl From<DiffFilesError> for ErrorKind {
    fn from(error: DiffFilesError) -> Self {
        match error {
            DiffFilesError::Common(common_error) => common_error.into(),
        }
    }
}
