// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::ErrorKind;

use upac_composefs::error::RepoError;

use upac_database::error::DatabaseError;

use upac_deploy::error::{ConfigDigestResolveError, DeployRecordsError, SysrootError};

use upac_orchestrator::error::PipelineError;
use upac_orchestrator::lock::LockError;

use crate::errors::{
    CommonError, common_error_from, config_digest_resolve_error_from, database_error_from, deploy_records_error_from,
    lock_error_from, pipeline_error_from, repo_error_from, sysroot_error_from,
};

#[cfg(test)]
#[path = "../../../tests/inline/unmutated_diff_config_error.rs"]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffConfigError {
    Common(CommonError),
    ConfigDigestNotFound(String),
}

common_error_from!(DiffConfigError);

pipeline_error_from!(DiffConfigError);

database_error_from!(DiffConfigError);

deploy_records_error_from!(DiffConfigError);

config_digest_resolve_error_from!(DiffConfigError);

repo_error_from!(DiffConfigError);

sysroot_error_from!(DiffConfigError);

lock_error_from!(DiffConfigError);

impl From<DiffConfigError> for ErrorKind {
    fn from(error: DiffConfigError) -> Self {
        match error {
            DiffConfigError::Common(common_error) => common_error.into(),
            DiffConfigError::ConfigDigestNotFound(_) => ErrorKind::NotFound,
        }
    }
}
