// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsStr;
use std::path::Path;

use composefs::generic_tree::{ImageError, LeafContent};
use composefs::tree::{Directory, Inode, Leaf};

use super::{Tree, TreeBuilder};

use crate::ObjectID;
use crate::error::RepoError;
use crate::layout::deployment::OVERLAY_OPAQUE_XATTR;

impl TreeBuilder {
    pub fn overlay(
        &mut self, path: impl AsRef<Path>, tree: &Tree, source_path: Option<&Path>,
    ) -> Result<(), RepoError> {
        let path = path.as_ref();
        let source_path = source_path.unwrap_or(Path::new(""));

        match tree.filesystem.root.get_directory(source_path.as_os_str()) {
            Ok(directory) => directory.entries().try_for_each(|(name, inode)| match inode {
                Inode::Directory(source) => {
                    self.overlay_subdirectory(&path.join(name), tree, &source_path.join(name), source)
                }
                Inode::Leaf(leaf_id, _) => self.overlay_leaf(&path.join(name), tree.filesystem.leaf(*leaf_id)),
            }),
            Err(ImageError::NotADirectory(_)) => self.overlay_leaf(path, tree.leaf(source_path)?),
            Err(error) => Err(error.into()),
        }
    }

    fn overlay_subdirectory(
        &mut self, path: &Path, tree: &Tree, source_path: &Path, source: &Directory<ObjectID>,
    ) -> Result<(), RepoError> {
        let mut stat = source.stat.clone();
        let is_opaque = stat
            .xattrs
            .remove(OsStr::new(OVERLAY_OPAQUE_XATTR))
            .is_some_and(|value| value.as_ref() == b"y");

        if !is_opaque && let Ok(directory) = self.filesystem.root.get_directory_mut(path.as_os_str()) {
            directory.stat = stat;
        } else {
            self.insert_dir(path, stat)?;
        }

        self.overlay(path, tree, Some(source_path))
    }

    fn overlay_leaf(&mut self, path: &Path, leaf: &Leaf<ObjectID>) -> Result<(), RepoError> {
        match leaf.content {
            LeafContent::CharacterDevice(0) => self.remove(path),
            _ => self.insert(path, leaf.content.clone(), leaf.stat.clone()),
        }
    }
}
