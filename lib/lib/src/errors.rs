// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::io::ErrorKind as IoErrorKind;

use upac_abi::error::ErrorKind;

use crate::composefs::error::RepoError;
use crate::database::error::{DatabaseError, DeployRecordError};
use crate::deploy::error::SysrootError;
use crate::lock::LockError;
use crate::plugin::decoder::error::DecoderError;
use crate::scripts::error::HookError;

macro_rules! common_error_from {
    ($name:ident) => {
        impl From<CommonError> for $name {
            fn from(error: CommonError) -> Self {
                $name::Common(error)
            }
        }
    };
}
pub(crate) use common_error_from;

macro_rules! database_error_from {
    ($name:ident) => {
        impl From<DatabaseError> for $name {
            fn from(error: DatabaseError) -> Self {
                $name::Common(CommonError::Database(error))
            }
        }
    };
}
pub(crate) use database_error_from;

macro_rules! sysroot_error_from {
    ($name:ident) => {
        impl From<SysrootError> for $name {
            fn from(error: SysrootError) -> Self {
                $name::Common(CommonError::Sysroot(error))
            }
        }
    };
}
pub(crate) use sysroot_error_from;

macro_rules! lock_error_from {
    ($name:ident) => {
        impl From<LockError> for $name {
            fn from(error: LockError) -> Self {
                $name::Common(CommonError::Lock(error))
            }
        }
    };
}
pub(crate) use lock_error_from;

macro_rules! deploy_record_error_from {
    ($name:ident) => {
        impl From<DeployRecordError> for $name {
            fn from(error: DeployRecordError) -> Self {
                $name::Common(CommonError::DeployRecord(error))
            }
        }
    };
}
pub(crate) use deploy_record_error_from;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommonError {
    OutOfMemory,
    Cancelled,
    AccessDenied,
    StageNotFound,
    StagePanicked,
    MissingResult,
    PipelineInvalid,
    RuntimeInit(IoErrorKind),
    Hook(HookError),
    Decoder(DecoderError),
    Repo(RepoError),
    Database(DatabaseError),
    Sysroot(SysrootError),
    Lock(LockError),
    DeployRecord(DeployRecordError),
}

impl From<CommonError> for ErrorKind {
    fn from(error: CommonError) -> Self {
        match error {
            CommonError::OutOfMemory => ErrorKind::OutOfMemory,
            CommonError::Cancelled => ErrorKind::Cancelled,
            CommonError::AccessDenied => ErrorKind::PermissionDenied,
            CommonError::StageNotFound
            | CommonError::StagePanicked
            | CommonError::MissingResult
            | CommonError::PipelineInvalid
            | CommonError::RuntimeInit(_) => ErrorKind::Unexpected,
            CommonError::Hook(hook_error) => hook_error.into(),
            CommonError::Decoder(decoder_error) => decoder_error.into(),
            CommonError::Repo(repo_error) => repo_error.into(),
            CommonError::Database(database_error) => database_error.into(),
            CommonError::Sysroot(sysroot_error) => sysroot_error.into(),
            CommonError::Lock(lock_error) => lock_error.into(),
            CommonError::DeployRecord(deploy_record_error) => deploy_record_error.into(),
        }
    }
}

impl From<HookError> for CommonError {
    fn from(error: HookError) -> Self {
        CommonError::Hook(error)
    }
}

impl From<DecoderError> for CommonError {
    fn from(error: DecoderError) -> Self {
        CommonError::Decoder(error)
    }
}

impl From<RepoError> for CommonError {
    fn from(error: RepoError) -> Self {
        CommonError::Repo(error)
    }
}

impl From<DatabaseError> for CommonError {
    fn from(error: DatabaseError) -> Self {
        CommonError::Database(error)
    }
}

impl From<SysrootError> for CommonError {
    fn from(error: SysrootError) -> Self {
        CommonError::Sysroot(error)
    }
}

impl From<LockError> for CommonError {
    fn from(error: LockError) -> Self {
        CommonError::Lock(error)
    }
}

impl From<DeployRecordError> for CommonError {
    fn from(error: DeployRecordError) -> Self {
        CommonError::DeployRecord(error)
    }
}
