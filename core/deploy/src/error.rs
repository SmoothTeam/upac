// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use nix::errno::Errno;

use rsmount::errors::MountInfoError;

use serde_json::Error as SerdeJsonError;

use upac_types::error::ErrorKind;

use upac_composefs::Digest;
use upac_composefs::error::RepoError;

use upac_database::error::DatabaseError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysrootError {
    MountInfoUnavailable,
    SysrootNotMounted,
    SysrootDirUnavailable,
    DeploysDirNotFound,
    RepoDirNotFound,
    Repository(RepoError),
    System(Errno),
}

impl From<MountInfoError> for SysrootError {
    fn from(_: MountInfoError) -> Self {
        SysrootError::MountInfoUnavailable
    }
}

impl From<IoError> for SysrootError {
    fn from(_: IoError) -> Self {
        SysrootError::SysrootDirUnavailable
    }
}

impl From<RepoError> for SysrootError {
    fn from(error: RepoError) -> Self {
        SysrootError::Repository(error)
    }
}

impl From<Errno> for SysrootError {
    fn from(errno: Errno) -> Self {
        SysrootError::System(errno)
    }
}

impl From<SysrootError> for ErrorKind {
    fn from(error: SysrootError) -> Self {
        match error {
            SysrootError::MountInfoUnavailable => ErrorKind::Unexpected,
            SysrootError::SysrootNotMounted => ErrorKind::NotFound,
            SysrootError::SysrootDirUnavailable => ErrorKind::NotFound,
            SysrootError::DeploysDirNotFound => ErrorKind::NotFound,
            SysrootError::RepoDirNotFound => ErrorKind::NotFound,
            SysrootError::Repository(error) => error.into(),
            SysrootError::System(_) => ErrorKind::Unexpected,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefixMetaError {
    NotFound,
    AccessDenied,
    MalformedJson,
    WriteFailed,
}

impl From<IoError> for PrefixMetaError {
    fn from(error: IoError) -> Self {
        match error.kind() {
            IoErrorKind::NotFound => PrefixMetaError::NotFound,
            IoErrorKind::PermissionDenied => PrefixMetaError::AccessDenied,
            _ => PrefixMetaError::WriteFailed,
        }
    }
}

impl From<SerdeJsonError> for PrefixMetaError {
    fn from(_: SerdeJsonError) -> Self {
        PrefixMetaError::MalformedJson
    }
}

impl From<PrefixMetaError> for ErrorKind {
    fn from(error: PrefixMetaError) -> Self {
        match error {
            PrefixMetaError::NotFound => ErrorKind::NotFound,
            PrefixMetaError::AccessDenied => ErrorKind::PermissionDenied,
            PrefixMetaError::MalformedJson => ErrorKind::ReadFailed,
            PrefixMetaError::WriteFailed => ErrorKind::WriteFailed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefixDeployError {
    ConfigNotFound(usize),
}

impl From<PrefixDeployError> for ErrorKind {
    fn from(error: PrefixDeployError) -> Self {
        match error {
            PrefixDeployError::ConfigNotFound(_) => ErrorKind::NotFound,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefixReadError {
    Sysroot(SysrootError),
    Repo(RepoError),
    Database(DatabaseError),
    Meta(PrefixMetaError),
    TransactionMissing,
}

impl From<SysrootError> for PrefixReadError {
    fn from(error: SysrootError) -> Self {
        PrefixReadError::Sysroot(error)
    }
}

impl From<IoError> for PrefixReadError {
    fn from(error: IoError) -> Self {
        PrefixReadError::Sysroot(error.into())
    }
}

impl From<RepoError> for PrefixReadError {
    fn from(error: RepoError) -> Self {
        PrefixReadError::Repo(error)
    }
}

impl From<DatabaseError> for PrefixReadError {
    fn from(error: DatabaseError) -> Self {
        PrefixReadError::Database(error)
    }
}

impl From<PrefixMetaError> for PrefixReadError {
    fn from(error: PrefixMetaError) -> Self {
        PrefixReadError::Meta(error)
    }
}

impl From<PrefixReadError> for ErrorKind {
    fn from(error: PrefixReadError) -> Self {
        match error {
            PrefixReadError::Sysroot(error) => error.into(),
            PrefixReadError::Repo(error) => error.into(),
            PrefixReadError::Database(error) => error.into(),
            PrefixReadError::Meta(error) => error.into(),
            PrefixReadError::TransactionMissing => ErrorKind::ReadFailed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefixCreateError {
    AlreadyExists(Digest),
    Meta(PrefixMetaError),
}

impl From<PrefixMetaError> for PrefixCreateError {
    fn from(error: PrefixMetaError) -> Self {
        PrefixCreateError::Meta(error)
    }
}

impl From<IoError> for PrefixCreateError {
    fn from(error: IoError) -> Self {
        PrefixCreateError::Meta(error.into())
    }
}

impl From<PrefixCreateError> for ErrorKind {
    fn from(error: PrefixCreateError) -> Self {
        match error {
            PrefixCreateError::AlreadyExists(_) => ErrorKind::InvalidEntry,
            PrefixCreateError::Meta(error) => error.into(),
        }
    }
}
