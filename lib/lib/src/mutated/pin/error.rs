// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::ErrorKind;

use crate::database::error::DeployRecordError;
use crate::deploy::error::SysrootError;
use crate::errors::{CommonError, common_error_from, deploy_record_error_from, lock_error_from, sysroot_error_from};
use crate::lock::LockError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinError {
    Common(CommonError),
}

common_error_from!(PinError);

sysroot_error_from!(PinError);

lock_error_from!(PinError);

deploy_record_error_from!(PinError);

impl From<PinError> for ErrorKind {
    fn from(error: PinError) -> Self {
        match error {
            PinError::Common(common_error) => common_error.into(),
        }
    }
}
