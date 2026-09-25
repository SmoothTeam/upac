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
#[path = "../../../tests/inline/mutated_files_error.rs"]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilesError {
    PackageNotFound,
    Common(CommonError),
}

common_error_from!(FilesError);

pipeline_error_from!(FilesError);

hook_error_from!(FilesError);

prune_error_from!(FilesError);

database_error_from!(FilesError);

sysroot_error_from!(FilesError);

lock_error_from!(FilesError);

boot_error_from!(FilesError);

repo_error_from!(FilesError);

deploy_record_error_from!(FilesError);

boot_plugin_error_from!(FilesError);

impl From<FilesError> for ErrorKind {
    fn from(error: FilesError) -> Self {
        match error {
            FilesError::PackageNotFound => ErrorKind::NotFound,
            FilesError::Common(common_error) => common_error.into(),
        }
    }
}
