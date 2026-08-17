// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::io::Error as IoError;

use anyhow::Error as AnyhowError;

use nix::errno::Errno;

use rsblkid::probe::{ProbeBuilderError, ProbeError};

use rsmount::errors::MountInfoError;

use upac_abi::error::ErrorKind;

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
            SysrootError::System(_) => ErrorKind::Unexpected,
        }
    }
}
