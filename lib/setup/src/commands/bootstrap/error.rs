// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use anyhow::Error as AnyhowError;

use nix::errno::Errno;

use upac::boot::error::BootError;
use upac::composefs::error::RepoError;
use upac::database::error::{DatabaseError, DeployRecordError};
use upac::errors::CommonError;
use upac::lock::LockError;
use upac::plugin::boot::error::BootPluginError;

use upac_abi::error::ErrorKind;

use crate::commands::partition::error::PartitionError;

#[cfg(test)]
#[path = "../../../tests/inline/bootstrap_error.rs"]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapError {
    Common(CommonError),
    Mount(Errno),
    Repo(RepoError),
    Database(DatabaseError),
    DeployRecord(DeployRecordError),
    Boot(BootError),
    BootPlugin(BootPluginError),
    Partition(PartitionError),
    Io(IoErrorKind),
    UnsupportedDeployFs,
    ComposefsSetupRootUnitNotFound,
    NoKernelFound,
    AmbiguousKernelVersion,
    UnknownInitramfsGenerator,
    InitramfsGeneratorFailed,
    Unexpected,
}

impl From<CommonError> for BootstrapError {
    fn from(error: CommonError) -> Self {
        BootstrapError::Common(error)
    }
}

impl From<LockError> for BootstrapError {
    fn from(error: LockError) -> Self {
        BootstrapError::Common(CommonError::Lock(error))
    }
}

impl From<Errno> for BootstrapError {
    fn from(errno: Errno) -> Self {
        BootstrapError::Mount(errno)
    }
}

impl From<RepoError> for BootstrapError {
    fn from(error: RepoError) -> Self {
        BootstrapError::Repo(error)
    }
}

impl From<DatabaseError> for BootstrapError {
    fn from(error: DatabaseError) -> Self {
        BootstrapError::Database(error)
    }
}

impl From<DeployRecordError> for BootstrapError {
    fn from(error: DeployRecordError) -> Self {
        BootstrapError::DeployRecord(error)
    }
}

impl From<BootError> for BootstrapError {
    fn from(error: BootError) -> Self {
        BootstrapError::Boot(error)
    }
}

impl From<BootPluginError> for BootstrapError {
    fn from(error: BootPluginError) -> Self {
        BootstrapError::BootPlugin(error)
    }
}

impl From<PartitionError> for BootstrapError {
    fn from(error: PartitionError) -> Self {
        BootstrapError::Partition(error)
    }
}

impl From<IoError> for BootstrapError {
    fn from(error: IoError) -> Self {
        BootstrapError::Io(error.kind())
    }
}

impl From<AnyhowError> for BootstrapError {
    fn from(_: AnyhowError) -> Self {
        BootstrapError::Unexpected
    }
}

impl From<BootstrapError> for ErrorKind {
    fn from(error: BootstrapError) -> Self {
        match error {
            BootstrapError::Common(common_error) => common_error.into(),
            BootstrapError::Mount(_) => ErrorKind::Unexpected,
            BootstrapError::Repo(repo_error) => repo_error.into(),
            BootstrapError::Database(database_error) => database_error.into(),
            BootstrapError::DeployRecord(deploy_record_error) => deploy_record_error.into(),
            BootstrapError::Boot(boot_error) => boot_error.into(),
            BootstrapError::BootPlugin(boot_plugin_error) => boot_plugin_error.into(),
            BootstrapError::Partition(partition_error) => partition_error.into(),
            BootstrapError::Io(kind) => match kind {
                IoErrorKind::NotFound => ErrorKind::NotFound,
                IoErrorKind::PermissionDenied => ErrorKind::PermissionDenied,
                IoErrorKind::AlreadyExists => ErrorKind::AlreadyExists,
                _ => ErrorKind::Unexpected,
            },
            BootstrapError::UnsupportedDeployFs => ErrorKind::InvalidEntry,
            BootstrapError::ComposefsSetupRootUnitNotFound => ErrorKind::NotFound,
            BootstrapError::NoKernelFound => ErrorKind::NotFound,
            BootstrapError::AmbiguousKernelVersion => ErrorKind::InvalidEntry,
            BootstrapError::UnknownInitramfsGenerator => ErrorKind::InvalidEntry,
            BootstrapError::InitramfsGeneratorFailed => ErrorKind::Unexpected,
            BootstrapError::Unexpected => ErrorKind::Unexpected,
        }
    }
}
