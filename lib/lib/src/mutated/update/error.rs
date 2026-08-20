// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
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
pub enum UpdateError {
    Common(CommonError),
}

common_error_from!(UpdateError);

database_error_from!(UpdateError);

sysroot_error_from!(UpdateError);

lock_error_from!(UpdateError);

boot_error_from!(UpdateError);

impl From<UpdateError> for ErrorKind {
    fn from(error: UpdateError) -> Self {
        match error {
            UpdateError::Common(common_error) => common_error.into(),
        }
    }
}
