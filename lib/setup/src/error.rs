// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use nix::errno::Errno;

use upac::composefs::error::RepoError;
use upac::database::error::{DatabaseError, DeployRecordError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupError {
    Mount(Errno),
    Repo(RepoError),
    Database(DatabaseError),
    DeployRecord(DeployRecordError),
    Io(IoErrorKind),
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
