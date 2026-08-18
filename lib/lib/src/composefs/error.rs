// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::io::Error as IoError;

use anyhow::Error as AnyhowError;

use composefs::generic_tree::ImageError;
use composefs::repository::RepositoryOpenError;

use hex::FromHexError;

use upac_abi::error::ErrorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoError {
    NotInitialized,
    Corrupted,
    AlgorithmMismatch,
    UnsupportedVersion,
    IncompatibleFeatures,
    NotFound,
    AccessDenied,
    InvalidPath,
    InvalidDigest,
    NotADirectory,
    IsADirectory,
    NotRegularFile,
    NotASymlink,
    Unexpected,
}

impl From<FromHexError> for RepoError {
    fn from(_: FromHexError) -> Self {
        RepoError::InvalidDigest
    }
}

impl From<IoError> for RepoError {
    fn from(error: IoError) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => RepoError::NotFound,
            std::io::ErrorKind::PermissionDenied => RepoError::AccessDenied,
            _ => RepoError::Unexpected,
        }
    }
}

impl From<RepositoryOpenError> for RepoError {
    fn from(error: RepositoryOpenError) -> Self {
        match error {
            RepositoryOpenError::MetadataMissing | RepositoryOpenError::OldFormatRepository => {
                RepoError::NotInitialized
            }
            RepositoryOpenError::MetadataInvalid(_) => RepoError::Corrupted,
            RepositoryOpenError::AlgorithmMismatch { .. } => RepoError::AlgorithmMismatch,
            RepositoryOpenError::UnsupportedVersion { .. } => RepoError::UnsupportedVersion,
            RepositoryOpenError::IncompatibleFeatures(_) => RepoError::IncompatibleFeatures,
            RepositoryOpenError::Io(io_error) => io_error.into(),
        }
    }
}

impl From<ImageError> for RepoError {
    fn from(error: ImageError) -> Self {
        match error {
            ImageError::InvalidFilename(_) => RepoError::InvalidPath,
            ImageError::NotFound(_) => RepoError::NotFound,
            ImageError::NotADirectory(_) => RepoError::NotADirectory,
            ImageError::IsADirectory(_) => RepoError::IsADirectory,
            ImageError::IsNotRegular(_) => RepoError::NotRegularFile,
            ImageError::LeafIdOutOfBounds(..) | ImageError::OrphanedLeaves(_) => RepoError::Unexpected,
        }
    }
}

impl From<AnyhowError> for RepoError {
    fn from(_: AnyhowError) -> Self {
        RepoError::Unexpected
    }
}

impl From<RepoError> for ErrorKind {
    fn from(error: RepoError) -> Self {
        match error {
            RepoError::NotInitialized => ErrorKind::NotInitialized,
            RepoError::Corrupted => ErrorKind::ReadFailed,
            RepoError::AlgorithmMismatch | RepoError::UnsupportedVersion | RepoError::IncompatibleFeatures => {
                ErrorKind::Unexpected
            }
            RepoError::NotFound => ErrorKind::NotFound,
            RepoError::AccessDenied => ErrorKind::PermissionDenied,
            RepoError::InvalidPath | RepoError::InvalidDigest => ErrorKind::InvalidPath,
            RepoError::NotADirectory | RepoError::IsADirectory | RepoError::NotRegularFile | RepoError::NotASymlink => {
                ErrorKind::InvalidEntry
            }
            RepoError::Unexpected => ErrorKind::Unexpected,
        }
    }
}
