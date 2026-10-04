// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::BTreeMap;
use std::path::Path;

use upac_types::diff::FileDiffKind;

use super::super::error::RepoError;
use super::{MergeResult, Tree};

impl Tree {
    pub fn merge(base: &Tree, new: &Tree, live: &Tree, allow_conflict_files: bool) -> Result<MergeResult, RepoError> {
        let user_changes = base.diff(live);
        let package_changes: BTreeMap<String, FileDiffKind> = base.diff(new).into_iter().collect();

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
                            tree.copy_leaf(Path::new(&format!("{path}.upac-new")), new, Path::new(&path))?;
                        }
                        conflicts.push(path.clone());
                    }

                    tree.copy_leaf(Path::new(&path), live, Path::new(&path))?;
                }
            }
        }

        Ok(MergeResult { tree, conflicts })
    }
}
