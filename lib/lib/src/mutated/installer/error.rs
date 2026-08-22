// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::error::ErrorKind;

use crate::boot::error::BootError;
use crate::composefs::error::RepoError;
use crate::database::error::{DatabaseError, DeployRecordError};
use crate::deploy::error::SysrootError;
use crate::errors::{
    CommonError, boot_error_from, common_error_from, database_error_from, deploy_record_error_from, lock_error_from,
    repo_error_from, sysroot_error_from,
};
use crate::lock::LockError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallError {
    Common(CommonError),
}

common_error_from!(InstallError);

database_error_from!(InstallError);

repo_error_from!(InstallError);

sysroot_error_from!(InstallError);

lock_error_from!(InstallError);

boot_error_from!(InstallError);

deploy_record_error_from!(InstallError);

impl From<InstallError> for ErrorKind {
    fn from(error: InstallError) -> Self {
        match error {
            InstallError::Common(common_error) => common_error.into(),
        }
    }
}
