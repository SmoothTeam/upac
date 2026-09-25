// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::ErrorKind;

use upac_boot_loader::entry::error::BootError;
use upac_boot_loader::error::BootPluginError;

use upac_composefs::error::RepoError;

use upac_database::error::DatabaseError;

use upac_deploy::error::{ConfigDigestResolveError, DeployRecordError, DeployRecordsError, PruneError, SysrootError};

use upac_hooks::error::HookError;

use upac_orchestrator::error::PipelineError;
use upac_orchestrator::lock::LockError;

use crate::errors::{
    CommonError, boot_error_from, boot_plugin_error_from, common_error_from, config_digest_resolve_error_from,
    database_error_from, deploy_record_error_from, deploy_records_error_from, hook_error_from, lock_error_from,
    pipeline_error_from, prune_error_from, repo_error_from, sysroot_error_from,
};

#[cfg(test)]
#[path = "../../../tests/inline/mutated_rollback_error.rs"]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollbackError {
    Common(CommonError),
    ConfigDigestNotFound(String),
}

common_error_from!(RollbackError);

pipeline_error_from!(RollbackError);

hook_error_from!(RollbackError);

prune_error_from!(RollbackError);

database_error_from!(RollbackError);

sysroot_error_from!(RollbackError);

lock_error_from!(RollbackError);

boot_error_from!(RollbackError);

boot_plugin_error_from!(RollbackError);

repo_error_from!(RollbackError);

deploy_record_error_from!(RollbackError);

deploy_records_error_from!(RollbackError);

config_digest_resolve_error_from!(RollbackError);

impl From<RollbackError> for ErrorKind {
    fn from(error: RollbackError) -> Self {
        match error {
            RollbackError::Common(common_error) => common_error.into(),
            RollbackError::ConfigDigestNotFound(_) => ErrorKind::NotFound,
        }
    }
}
