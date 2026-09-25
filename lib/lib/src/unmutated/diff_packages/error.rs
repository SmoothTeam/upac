// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::ErrorKind;

use upac_composefs::error::RepoError;

use upac_database::error::DatabaseError;

use upac_deploy::error::SysrootError;

use upac_orchestrator::error::PipelineError;
use upac_orchestrator::lock::LockError;

use crate::errors::{
    CommonError, common_error_from, database_error_from, lock_error_from, pipeline_error_from, repo_error_from,
    sysroot_error_from,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffPackagesError {
    Common(CommonError),
}

common_error_from!(DiffPackagesError);

pipeline_error_from!(DiffPackagesError);

database_error_from!(DiffPackagesError);

repo_error_from!(DiffPackagesError);

sysroot_error_from!(DiffPackagesError);

lock_error_from!(DiffPackagesError);

impl From<DiffPackagesError> for ErrorKind {
    fn from(error: DiffPackagesError) -> Self {
        match error {
            DiffPackagesError::Common(common_error) => common_error.into(),
        }
    }
}
