// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::path::Path;

use composefs::fsverity::Sha256HashValue;
use composefs::repository::Repository;
use nix::fcntl::AT_FDCWD;

use crate::composefs::error::RepoError;

pub type ObjectID = Sha256HashValue;

pub fn open(path: &Path) -> Result<Repository<ObjectID>, RepoError> {
    Ok(Repository::open_path(AT_FDCWD, path)?)
}
