// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::collections::BTreeSet;
use std::path::Path;

use composefs::tree::FileSystem;

use upac_abi::FileDiffKind;

use crate::composefs::diff::TreeDiff;
use crate::composefs::error::RepoError;
use crate::composefs::file::FileHandle;
use crate::composefs::repository::ObjectID;

pub struct MergeResult {
    pub tree: FileSystem<ObjectID>,
    pub conflicts: Vec<String>,
}

pub fn merge_config(
    base: &FileSystem<ObjectID>, new: &FileSystem<ObjectID>, live: &FileSystem<ObjectID>,
) -> Result<MergeResult, RepoError> {
    let user_changes = TreeDiff::run(base, live);
    let package_changed: BTreeSet<String> = TreeDiff::run(base, new).into_iter().map(|(path, _)| path).collect();

    let mut tree = new.clone();
    let mut conflicts = Vec::new();

    for (path, kind) in user_changes {
        let conflict = package_changed.contains(&path);

        match kind {
            FileDiffKind::Removed => {
                if conflict {
                    conflicts.push(path);
                } else {
                    FileHandle::new(&path).remove_in_tree(&mut tree)?;
                }
            }
            FileDiffKind::Added | FileDiffKind::Modified => {
                if conflict {
                    FileHandle::new(format!("{path}.upac-new")).copy_from_tree(&mut tree, new, Path::new(&path))?;
                    conflicts.push(path.clone());
                }

                FileHandle::new(&path).copy_from_tree(&mut tree, live, Path::new(&path))?;
            }
        }
    }

    Ok(MergeResult { tree, conflicts })
}
