// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::io::ErrorKind as IoErrorKind;

use upac_abi::error::ErrorKind;

use crate::boot::error::BootError;
use crate::composefs::error::RepoError;
use crate::database::error::{DatabaseError, DeployRecordError};
use crate::deploy::error::SysrootError;
use crate::lock::LockError;
use crate::plugin::boot::error::BootPluginError;
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

macro_rules! repo_error_from {
    ($name:ident) => {
        impl From<RepoError> for $name {
            fn from(error: RepoError) -> Self {
                $name::Common(CommonError::Repo(error))
            }
        }
    };
}
pub(crate) use repo_error_from;

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

macro_rules! boot_error_from {
    ($name:ident) => {
        impl From<BootError> for $name {
            fn from(error: BootError) -> Self {
                $name::Common(CommonError::Boot(error))
            }
        }
    };
}
pub(crate) use boot_error_from;

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

macro_rules! deploy_records_error_from {
    ($name:ident) => {
        impl From<DeployRecordsError> for $name {
            fn from(error: DeployRecordsError) -> Self {
                match error {
                    DeployRecordsError::Sysroot(error) => $name::Common(CommonError::Sysroot(error)),
                    DeployRecordsError::DeployRecord(error) => $name::Common(CommonError::DeployRecord(error)),
                }
            }
        }
    };
}
pub(crate) use deploy_records_error_from;

macro_rules! config_digest_resolve_error_from {
    ($name:ident) => {
        impl From<ConfigDigestResolveError> for $name {
            fn from(error: ConfigDigestResolveError) -> Self {
                match error {
                    ConfigDigestResolveError::Records(records_error) => records_error.into(),
                    ConfigDigestResolveError::NotFound(config_digest) => $name::ConfigDigestNotFound(config_digest),
                }
            }
        }
    };
}
pub(crate) use config_digest_resolve_error_from;

macro_rules! regex_error_from {
    ($name:ident) => {
        impl From<regex::Error> for $name {
            fn from(error: regex::Error) -> Self {
                $name::InvalidSearchPattern(error.to_string())
            }
        }
    };
}
pub(crate) use regex_error_from;

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
    Boot(BootError),
    BootPlugin(BootPluginError),
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
            CommonError::Boot(boot_error) => boot_error.into(),
            CommonError::BootPlugin(boot_plugin_error) => boot_plugin_error.into(),
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

impl From<BootPluginError> for CommonError {
    fn from(error: BootPluginError) -> Self {
        CommonError::BootPlugin(error)
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

impl From<BootError> for CommonError {
    fn from(error: BootError) -> Self {
        CommonError::Boot(error)
    }
}
