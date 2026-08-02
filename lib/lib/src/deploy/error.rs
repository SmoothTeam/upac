use std::io::Error as IoError;

use nix::errno::Errno;
use rsblkid::cache::{CacheBuilderError, CacheError};
use rsmount::errors::MountInfoError;
use upac_abi::error::ErrorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysrootError {
    MountInfoUnavailable,
    RootDeviceNotFound,
    CacheUnavailable,
    UuidNotFound,
    CanonicalDeviceNotFound,
    SysrootDirUnavailable,
    DeploysDirNotFound,
    RepoDirNotFound,
    System(Errno),
}

impl From<MountInfoError> for SysrootError {
    fn from(_: MountInfoError) -> Self {
        SysrootError::MountInfoUnavailable
    }
}

impl From<CacheBuilderError> for SysrootError {
    fn from(_: CacheBuilderError) -> Self {
        SysrootError::CacheUnavailable
    }
}

impl From<CacheError> for SysrootError {
    fn from(_: CacheError) -> Self {
        SysrootError::CacheUnavailable
    }
}

impl From<uuid::Error> for SysrootError {
    fn from(_: uuid::Error) -> Self {
        SysrootError::UuidNotFound
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

impl From<SysrootError> for ErrorKind {
    fn from(error: SysrootError) -> Self {
        match error {
            SysrootError::MountInfoUnavailable => ErrorKind::Unexpected,
            SysrootError::RootDeviceNotFound => ErrorKind::NotFound,
            SysrootError::CacheUnavailable => ErrorKind::Unexpected,
            SysrootError::UuidNotFound => ErrorKind::NotFound,
            SysrootError::CanonicalDeviceNotFound => ErrorKind::NotFound,
            SysrootError::SysrootDirUnavailable => ErrorKind::NotFound,
            SysrootError::DeploysDirNotFound => ErrorKind::NotFound,
            SysrootError::RepoDirNotFound => ErrorKind::NotFound,
            SysrootError::System(_) => ErrorKind::Unexpected,
        }
    }
}
