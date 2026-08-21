// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs::{File, Metadata, read_dir, read_link};
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};

use composefs::repository::{ImportContext, Repository};
use composefs::tree::FileSystem;

use crate::composefs::error::RepoError;
use crate::composefs::file::{FileHandle, stat_from_metadata};
use crate::composefs::repository::ObjectID;

const OVERLAY_OPAQUE_XATTR: &str = "trusted.overlay.opaque";

pub fn apply_overlay_upper(
    repository: &Repository<ObjectID>, tree: &mut FileSystem<ObjectID>, upper_dir: &Path, ctx: &mut ImportContext,
) -> Result<(), RepoError> {
    apply_overlay_prefix(repository, tree, &PathBuf::new(), upper_dir, ctx)
}

fn apply_overlay_prefix(
    repository: &Repository<ObjectID>, tree: &mut FileSystem<ObjectID>, tree_prefix: &Path, upper_dir: &Path,
    ctx: &mut ImportContext,
) -> Result<(), RepoError> {
    for entry in read_dir(upper_dir)? {
        let entry = entry?;
        let source_path = entry.path();
        let metadata = entry.metadata()?;
        let child_prefix = tree_prefix.join(entry.file_name());
        let child = FileHandle::new(&child_prefix);

        if is_whiteout(&metadata) {
            child.remove_in_tree(tree)?;
            continue;
        }

        let stat = stat_from_metadata(&metadata);

        if metadata.is_dir() {
            if is_opaque(&source_path)? || child.stat_in_tree(tree).is_err() {
                child.remove_in_tree(tree)?;
                child.insert_in_tree(tree, stat)?;
            } else {
                child.update_in_tree(tree, stat)?;
            }

            apply_overlay_prefix(repository, tree, &child_prefix, &source_path, ctx)?;
        } else if metadata.is_symlink() {
            child.remove_in_tree(tree)?;
            child.symlink_in_tree(tree, read_link(&source_path)?, stat)?;
        } else {
            child.remove_in_tree(tree)?;
            child.insert_file(repository, tree, &File::open(&source_path)?, stat, ctx)?;
        }
    }

    Ok(())
}

fn is_whiteout(metadata: &Metadata) -> bool {
    metadata.file_type().is_char_device() && metadata.rdev() == 0
}

fn is_opaque(path: &Path) -> Result<bool, RepoError> {
    Ok(xattr::get(path, OVERLAY_OPAQUE_XATTR)?.as_deref() == Some(b"y"))
}
