// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::ErrorKind;

use upac_database::error::DatabaseError;

use upac_deploy::error::{DeployRecordError, SysrootError};

use upac_orchestrator::error::PipelineError;
use upac_orchestrator::lock::LockError;

use crate::errors::{
    CommonError, common_error_from, database_error_from, deploy_record_error_from, lock_error_from,
    pipeline_error_from, sysroot_error_from,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListConfigError {
    Common(CommonError),
}

common_error_from!(ListConfigError);

pipeline_error_from!(ListConfigError);

database_error_from!(ListConfigError);

deploy_record_error_from!(ListConfigError);

sysroot_error_from!(ListConfigError);

lock_error_from!(ListConfigError);

impl From<ListConfigError> for ErrorKind {
    fn from(error: ListConfigError) -> Self {
        match error {
            ListConfigError::Common(common_error) => common_error.into(),
        }
    }
}
