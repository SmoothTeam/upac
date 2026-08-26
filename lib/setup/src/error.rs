// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use anyhow::Error as AnyhowError;

use gptman::Error as GptError;
use gptman::linux::BlockError as GptBlockError;

use nix::errno::Errno;

use toml::de::Error as TomlError;

use upac::composefs::error::RepoError;
use upac::database::error::{DatabaseError, DeployRecordError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupError {
    Mount(Errno),
    Repo(RepoError),
    Database(DatabaseError),
    DeployRecord(DeployRecordError),
    Io(IoErrorKind),
    MetaMalformed,
    NoSpaceLeft,
    NotBlockDevice,
    MkfsFailed,
    Unexpected,
}

impl From<Errno> for SetupError {
    fn from(errno: Errno) -> Self {
        SetupError::Mount(errno)
    }
}

impl From<RepoError> for SetupError {
    fn from(error: RepoError) -> Self {
        SetupError::Repo(error)
    }
}

impl From<DatabaseError> for SetupError {
    fn from(error: DatabaseError) -> Self {
        SetupError::Database(error)
    }
}

impl From<DeployRecordError> for SetupError {
    fn from(error: DeployRecordError) -> Self {
        SetupError::DeployRecord(error)
    }
}

impl From<IoError> for SetupError {
    fn from(error: IoError) -> Self {
        SetupError::Io(error.kind())
    }
}

impl From<TomlError> for SetupError {
    fn from(_: TomlError) -> Self {
        SetupError::MetaMalformed
    }
}

impl From<GptError> for SetupError {
    fn from(error: GptError) -> Self {
        match error {
            GptError::Io(io_error) => SetupError::Io(io_error.kind()),
            GptError::NoSpaceLeft => SetupError::NoSpaceLeft,
            _ => SetupError::Unexpected,
        }
    }
}

impl From<GptBlockError> for SetupError {
    fn from(error: GptBlockError) -> Self {
        match error {
            GptBlockError::Metadata(io_error) => SetupError::Io(io_error.kind()),
            GptBlockError::NotBlock => SetupError::NotBlockDevice,
            _ => SetupError::Unexpected,
        }
    }
}

impl From<AnyhowError> for SetupError {
    fn from(_: AnyhowError) -> Self {
        SetupError::Unexpected
    }
}
