// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::error::ErrorKind;

use crate::composefs::error::RepoError;
use crate::database::error::{DatabaseError, DeployRecordError};
use crate::deploy::error::SysrootError;
use crate::errors::{
    CommonError, common_error_from, database_error_from, deploy_record_error_from, lock_error_from, repo_error_from,
    sysroot_error_from,
};
use crate::lock::LockError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GcError {
    Common(CommonError),
}

common_error_from!(GcError);

database_error_from!(GcError);

repo_error_from!(GcError);

sysroot_error_from!(GcError);

lock_error_from!(GcError);

deploy_record_error_from!(GcError);

impl From<GcError> for ErrorKind {
    fn from(error: GcError) -> Self {
        match error {
            GcError::Common(common_error) => common_error.into(),
        }
    }
}
