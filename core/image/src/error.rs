// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_composefs::error::RepoError;

use upac_database::error::DatabaseError;

use upac_types::error::ErrorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageError {
    Tree(RepoError),
    Database(DatabaseError),
}

impl From<RepoError> for ImageError {
    fn from(error: RepoError) -> Self {
        ImageError::Tree(error)
    }
}

impl From<DatabaseError> for ImageError {
    fn from(error: DatabaseError) -> Self {
        ImageError::Database(error)
    }
}

impl From<ImageError> for ErrorKind {
    fn from(error: ImageError) -> Self {
        match error {
            ImageError::Tree(error) => error.into(),
            ImageError::Database(error) => error.into(),
        }
    }
}
