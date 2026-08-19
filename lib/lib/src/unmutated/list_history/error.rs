// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::error::ErrorKind;

use crate::database::error::{DatabaseError, DeployRecordsError};
use crate::deploy::error::SysrootError;
use crate::errors::{
    CommonError, common_error_from, database_error_from, deploy_records_error_from, lock_error_from, sysroot_error_from,
};
use crate::lock::LockError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListHistoryError {
    Common(CommonError),
}

common_error_from!(ListHistoryError);

database_error_from!(ListHistoryError);

deploy_records_error_from!(ListHistoryError);

sysroot_error_from!(ListHistoryError);

lock_error_from!(ListHistoryError);

impl From<ListHistoryError> for ErrorKind {
    fn from(error: ListHistoryError) -> Self {
        match error {
            ListHistoryError::Common(common_error) => common_error.into(),
        }
    }
}
