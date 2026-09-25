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
pub enum DiffPrefixError {
    Common(CommonError),
}

common_error_from!(DiffPrefixError);

pipeline_error_from!(DiffPrefixError);

database_error_from!(DiffPrefixError);

repo_error_from!(DiffPrefixError);

sysroot_error_from!(DiffPrefixError);

lock_error_from!(DiffPrefixError);

impl From<DiffPrefixError> for ErrorKind {
    fn from(error: DiffPrefixError) -> Self {
        match error {
            DiffPrefixError::Common(common_error) => common_error.into(),
        }
    }
}
