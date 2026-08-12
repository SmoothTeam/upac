// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::error::ErrorKind;

use crate::composefs::error::RepoError;
use crate::database::error::DatabaseError;
use crate::deploy::error::SysrootError;
use crate::errors::{
    CommonError, common_error_from, database_error_from, lock_error_from, repo_error_from, sysroot_error_from,
};
use crate::lock::LockError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchMetaError {
    Common(CommonError),
}

common_error_from!(SearchMetaError);

database_error_from!(SearchMetaError);

repo_error_from!(SearchMetaError);

sysroot_error_from!(SearchMetaError);

lock_error_from!(SearchMetaError);

impl From<SearchMetaError> for ErrorKind {
    fn from(error: SearchMetaError) -> Self {
        match error {
            SearchMetaError::Common(common_error) => common_error.into(),
        }
    }
}
