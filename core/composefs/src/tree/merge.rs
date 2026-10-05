// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use upac_types::diff::FileDiffKind;

use super::super::error::RepoError;
use super::super::layout::merge::UPAC_NEW_SUFFIX;
use super::{MergeResult, Tree};

impl Tree {
    pub fn merge(base: &Tree, new: &Tree, live: &Tree, allow_conflict_files: bool) -> Result<MergeResult, RepoError> {
        let user_changes = base.diff(live);
        let package_changes = base.diff(new);

        let mut tree = new.clone();
        let mut conflicts = Vec::new();

        for (path, kind) in user_changes {
            let package_change = package_changes.get(&path);

            match kind {
                FileDiffKind::Removed => match package_change {
                    Some(FileDiffKind::Added | FileDiffKind::Modified) => conflicts.push(path),
                    Some(FileDiffKind::Removed) => {}
                    None => tree.remove(&path)?,
                },
                FileDiffKind::Added | FileDiffKind::Modified => {
                    if let Some(FileDiffKind::Added | FileDiffKind::Modified) = package_change {
                        if allow_conflict_files {
                            tree.copy_leaf(&Self::conflict_copy_path(&path), new, &path)?;
                        }
                        conflicts.push(path.clone());
                    }

                    tree.copy_leaf(&path, live, &path)?;
                }
            }
        }

        Ok(MergeResult { tree, conflicts })
    }

    fn conflict_copy_path(path: &Path) -> PathBuf {
        let mut copy_path = OsString::from(path);
        copy_path.push(UPAC_NEW_SUFFIX);

        PathBuf::from(copy_path)
    }
}
