// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::ErrorKind;

use upac_database::error::DatabaseError;

use upac_deploy::error::{DeployRecordsError, SysrootError};

use upac_orchestrator::error::PipelineError;
use upac_orchestrator::lock::LockError;

use crate::errors::{
    CommonError, common_error_from, database_error_from, deploy_records_error_from, lock_error_from,
    pipeline_error_from, sysroot_error_from,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListHistoryError {
    Common(CommonError),
}

common_error_from!(ListHistoryError);

pipeline_error_from!(ListHistoryError);

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
