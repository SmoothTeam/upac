// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::Error as IoError;

use anyhow::Error as AnyhowError;

use nix::errno::Errno;

use rsblkid::probe::{ProbeBuilderError, ProbeError};

use rsmount::errors::MountInfoError;

use serde_json::Error as SerdeJsonError;

use upac_abi::error::ErrorKind;

use upac_composefs::error::RepoError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysrootError {
    MountInfoUnavailable,
    RootDeviceNotFound,
    CanonicalDeviceNotFound,
    SysrootDirUnavailable,
    DeploysDirNotFound,
    RepoDirNotFound,
    ProbeUnavailable,
    FilesystemTypeNotFound,
    CurrentPrefixDigestNotFound,
    EspNotFound,
    System(Errno),
}

impl From<MountInfoError> for SysrootError {
    fn from(_: MountInfoError) -> Self {
        SysrootError::MountInfoUnavailable
    }
}

impl From<ProbeBuilderError> for SysrootError {
    fn from(_: ProbeBuilderError) -> Self {
        SysrootError::ProbeUnavailable
    }
}

impl From<ProbeError> for SysrootError {
    fn from(_: ProbeError) -> Self {
        SysrootError::ProbeUnavailable
    }
}

impl From<IoError> for SysrootError {
    fn from(_: IoError) -> Self {
        SysrootError::SysrootDirUnavailable
    }
}

impl From<Errno> for SysrootError {
    fn from(errno: Errno) -> Self {
        SysrootError::System(errno)
    }
}

impl From<AnyhowError> for SysrootError {
    fn from(_: AnyhowError) -> Self {
        SysrootError::CurrentPrefixDigestNotFound
    }
}

impl From<SysrootError> for ErrorKind {
    fn from(error: SysrootError) -> Self {
        match error {
            SysrootError::MountInfoUnavailable => ErrorKind::Unexpected,
            SysrootError::RootDeviceNotFound => ErrorKind::NotFound,
            SysrootError::CanonicalDeviceNotFound => ErrorKind::NotFound,
            SysrootError::SysrootDirUnavailable => ErrorKind::NotFound,
            SysrootError::DeploysDirNotFound => ErrorKind::NotFound,
            SysrootError::RepoDirNotFound => ErrorKind::NotFound,
            SysrootError::ProbeUnavailable => ErrorKind::Unexpected,
            SysrootError::FilesystemTypeNotFound => ErrorKind::NotFound,
            SysrootError::CurrentPrefixDigestNotFound => ErrorKind::NotFound,
            SysrootError::EspNotFound => ErrorKind::NotFound,
            SysrootError::System(_) => ErrorKind::Unexpected,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeployRecordError {
    NotFound,
    AccessDenied,
    MalformedJson,
    InvalidField,
    WriteFailed,
}

impl From<IoError> for DeployRecordError {
    fn from(error: IoError) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => DeployRecordError::NotFound,
            std::io::ErrorKind::PermissionDenied => DeployRecordError::AccessDenied,
            _ => DeployRecordError::WriteFailed,
        }
    }
}

impl From<SerdeJsonError> for DeployRecordError {
    fn from(_: SerdeJsonError) -> Self {
        DeployRecordError::MalformedJson
    }
}

impl From<DeployRecordError> for ErrorKind {
    fn from(error: DeployRecordError) -> Self {
        match error {
            DeployRecordError::NotFound => ErrorKind::NotFound,
            DeployRecordError::AccessDenied => ErrorKind::PermissionDenied,
            DeployRecordError::MalformedJson | DeployRecordError::InvalidField => ErrorKind::ReadFailed,
            DeployRecordError::WriteFailed => ErrorKind::WriteFailed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeployRecordsError {
    Sysroot(SysrootError),
    DeployRecord(DeployRecordError),
}

impl From<SysrootError> for DeployRecordsError {
    fn from(error: SysrootError) -> Self {
        DeployRecordsError::Sysroot(error)
    }
}

impl From<DeployRecordError> for DeployRecordsError {
    fn from(error: DeployRecordError) -> Self {
        DeployRecordsError::DeployRecord(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigDigestResolveError {
    Records(DeployRecordsError),
    NotFound(String),
}

impl From<DeployRecordsError> for ConfigDigestResolveError {
    fn from(error: DeployRecordsError) -> Self {
        ConfigDigestResolveError::Records(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PruneError {
    Records(DeployRecordsError),
    Repo(RepoError),
}

impl From<DeployRecordsError> for PruneError {
    fn from(error: DeployRecordsError) -> Self {
        PruneError::Records(error)
    }
}

impl From<RepoError> for PruneError {
    fn from(error: RepoError) -> Self {
        PruneError::Repo(error)
    }
}
