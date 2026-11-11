// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_composefs::error::RepoError;

use upac_types::error::ErrorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseError {
    InvalidNodeId,
    InvalidMeta,
    InvalidTriggers,
    InvalidTransaction,
    UnsupportedNode,
    NodeKindMismatch,
    Tree(RepoError),
}

impl From<RepoError> for DatabaseError {
    fn from(error: RepoError) -> Self {
        DatabaseError::Tree(error)
    }
}

impl From<DatabaseError> for ErrorKind {
    fn from(error: DatabaseError) -> Self {
        match error {
            DatabaseError::InvalidNodeId
            | DatabaseError::InvalidMeta
            | DatabaseError::InvalidTriggers
            | DatabaseError::InvalidTransaction
            | DatabaseError::UnsupportedNode
            | DatabaseError::NodeKindMismatch => ErrorKind::InvalidEntry,
            DatabaseError::Tree(error) => error.into(),
        }
    }
}
