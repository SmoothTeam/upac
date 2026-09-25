// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::ErrorKind;

use upac_composefs::error::RepoError;

use upac_database::error::DatabaseError;

use upac_deploy::error::{DeployRecordError, SysrootError};

use upac_orchestrator::error::PipelineError;
use upac_orchestrator::lock::LockError;

use crate::errors::{
    CommonError, common_error_from, database_error_from, deploy_record_error_from, lock_error_from,
    pipeline_error_from, repo_error_from, sysroot_error_from,
};

#[cfg(test)]
#[path = "../../../tests/inline/unmutated_diff_error.rs"]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffError {
    Common(CommonError),
    ConfigDigestNotFound(String),
}

common_error_from!(DiffError);

pipeline_error_from!(DiffError);

database_error_from!(DiffError);

deploy_record_error_from!(DiffError);

repo_error_from!(DiffError);

sysroot_error_from!(DiffError);

lock_error_from!(DiffError);

impl From<DiffError> for ErrorKind {
    fn from(error: DiffError) -> Self {
        match error {
            DiffError::Common(common_error) => common_error.into(),
            DiffError::ConfigDigestNotFound(_) => ErrorKind::NotFound,
        }
    }
}
