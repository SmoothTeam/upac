// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::ErrorKind;

use upac_boot_loader::entry::error::BootError;
use upac_boot_loader::error::BootPluginError;

use upac_composefs::error::RepoError;

use upac_database::error::DatabaseError;

use upac_deploy::error::{DeployRecordError, PruneError, SysrootError};

use upac_hooks::error::HookError;

use upac_orchestrator::error::PipelineError;
use upac_orchestrator::lock::LockError;

use crate::errors::{
    CommonError, boot_error_from, boot_plugin_error_from, common_error_from, database_error_from,
    deploy_record_error_from, hook_error_from, lock_error_from, pipeline_error_from, prune_error_from, repo_error_from,
    sysroot_error_from,
};

#[cfg(test)]
#[path = "../../../tests/inline/mutated_update_error.rs"]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateError {
    PackageNotFound,
    DowngradeNotAllowed,
    Common(CommonError),
}

common_error_from!(UpdateError);

pipeline_error_from!(UpdateError);

hook_error_from!(UpdateError);

prune_error_from!(UpdateError);

database_error_from!(UpdateError);

sysroot_error_from!(UpdateError);

lock_error_from!(UpdateError);

boot_error_from!(UpdateError);

repo_error_from!(UpdateError);

deploy_record_error_from!(UpdateError);

boot_plugin_error_from!(UpdateError);

impl From<UpdateError> for ErrorKind {
    fn from(error: UpdateError) -> Self {
        match error {
            UpdateError::PackageNotFound => ErrorKind::NotFound,
            UpdateError::DowngradeNotAllowed => ErrorKind::InvalidEntry,
            UpdateError::Common(common_error) => common_error.into(),
        }
    }
}
