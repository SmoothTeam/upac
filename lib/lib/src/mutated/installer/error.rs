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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallError {
    Common(CommonError),
}

common_error_from!(InstallError);

pipeline_error_from!(InstallError);

hook_error_from!(InstallError);

prune_error_from!(InstallError);

database_error_from!(InstallError);

repo_error_from!(InstallError);

sysroot_error_from!(InstallError);

lock_error_from!(InstallError);

boot_error_from!(InstallError);

boot_plugin_error_from!(InstallError);

deploy_record_error_from!(InstallError);

impl From<InstallError> for ErrorKind {
    fn from(error: InstallError) -> Self {
        match error {
            InstallError::Common(common_error) => common_error.into(),
        }
    }
}
