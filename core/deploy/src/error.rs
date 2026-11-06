// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use nix::errno::Errno;

use procfs::ProcError;

use upac_types::error::ErrorKind;

use upac_composefs::error::RepoError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysrootError {
    MountInfoUnavailable,
    SysrootNotMounted,
    SysrootDirUnavailable,
    DeploysDirNotFound,
    RepoDirNotFound,
    InvalidDeploy,
    PointerNotFound,
    InvalidPointer,
    Deploy(DeployError),
    Repository(RepoError),
    System(Errno),
}

impl From<ProcError> for SysrootError {
    fn from(_: ProcError) -> Self {
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

impl From<DeployError> for SysrootError {
    fn from(error: DeployError) -> Self {
        SysrootError::Deploy(error)
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
            SysrootError::InvalidDeploy => ErrorKind::InvalidEntry,
            SysrootError::PointerNotFound => ErrorKind::NotFound,
            SysrootError::InvalidPointer => ErrorKind::InvalidEntry,
            SysrootError::Deploy(error) => error.into(),
            SysrootError::Repository(error) => error.into(),
            SysrootError::System(_) => ErrorKind::Unexpected,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeployError {
    NotFound,
    AlreadyExists,
    InvalidSnapshot,
    Io(IoErrorKind),
}

impl From<IoError> for DeployError {
    fn from(error: IoError) -> Self {
        DeployError::Io(error.kind())
    }
}

impl From<DeployError> for ErrorKind {
    fn from(error: DeployError) -> Self {
        match error {
            DeployError::NotFound => ErrorKind::NotFound,
            DeployError::AlreadyExists => ErrorKind::AlreadyExists,
            DeployError::InvalidSnapshot => ErrorKind::InvalidEntry,
            DeployError::Io(kind) => ErrorKind::from(IoError::from(kind)),
        }
    }
}
