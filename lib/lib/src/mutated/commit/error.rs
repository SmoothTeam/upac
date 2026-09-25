// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::ErrorKind;

use upac_composefs::error::RepoError;

use upac_database::error::DatabaseError;

use upac_deploy::error::{DeployRecordError, PruneError, SysrootError};

use upac_hooks::error::HookError;

use upac_orchestrator::error::PipelineError;
use upac_orchestrator::lock::LockError;

use crate::errors::{
    CommonError, common_error_from, database_error_from, deploy_record_error_from, hook_error_from, lock_error_from,
    pipeline_error_from, prune_error_from, repo_error_from, sysroot_error_from,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitError {
    Common(CommonError),
}

common_error_from!(CommitError);

pipeline_error_from!(CommitError);

hook_error_from!(CommitError);

prune_error_from!(CommitError);

database_error_from!(CommitError);

sysroot_error_from!(CommitError);

lock_error_from!(CommitError);

repo_error_from!(CommitError);

deploy_record_error_from!(CommitError);

impl From<CommitError> for ErrorKind {
    fn from(error: CommitError) -> Self {
        match error {
            CommitError::Common(common_error) => common_error.into(),
        }
    }
}
