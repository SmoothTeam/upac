// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use composefs::generic_tree::{LeafContent, LeafId};
use composefs::tree::{Directory, Inode, Leaf};

use super::Tree;
use super::builder::TreeBuilder;

use crate::ObjectID;
use crate::error::RepoError;
use crate::layout::deployment::OVERLAY_OPAQUE_XATTR;

pub struct OverlayBuilder<'tree> {
    lower: &'tree Tree,
    upper: &'tree Tree,
}

impl<'tree> OverlayBuilder<'tree> {
    pub fn new(lower: &'tree Tree, upper: &'tree Tree) -> Self {
        Self { lower, upper }
    }

    pub fn commit(self) -> Result<Tree, RepoError> {
        let mut builder = self.lower.edit();
        let mut pending = vec![(PathBuf::new(), &self.upper.filesystem.root)];
        let mut inserted_leaves: HashMap<LeafId, PathBuf> = HashMap::new();

        while let Some((path, directory)) = pending.pop() {
            for (name, inode) in directory.entries() {
                let child = path.join(name);

                match inode {
                    Inode::Directory(subdirectory) => {
                        self.overlay_directory(&mut builder, subdirectory, &child)?;
                        pending.push((child, subdirectory));
                    }
                    Inode::Leaf(leaf_id, _) => match inserted_leaves.get(leaf_id) {
                        Some(original) => builder.insert_hard_link(&child, original)?,
                        None => {
                            self.overlay_leaf(&mut builder, &child, self.upper.filesystem.leaf(*leaf_id))?;
                            inserted_leaves.insert(*leaf_id, child);
                        }
                    },
                }
            }
        }

        Ok(builder.commit())
    }

    fn overlay_directory(
        &self, builder: &mut TreeBuilder, subdirectory: &Directory<ObjectID>, path: &Path,
    ) -> Result<(), RepoError> {
        let mut stat = subdirectory.stat.clone();
        let is_opaque = stat
            .xattrs
            .remove(OsStr::new(OVERLAY_OPAQUE_XATTR))
            .is_some_and(|value| value.as_ref() == b"y");

        if !is_opaque && let Ok(directory) = builder.filesystem.root.get_directory_mut(path.as_os_str()) {
            directory.stat = stat;
        } else {
            builder.insert_dir(path, stat)?;
        }

        Ok(())
    }

    fn overlay_leaf(&self, builder: &mut TreeBuilder, path: &Path, leaf: &Leaf<ObjectID>) -> Result<(), RepoError> {
        match leaf.content {
            LeafContent::CharacterDevice(0) => builder.remove(path),
            _ => builder.insert(path, leaf.content.clone(), leaf.stat.clone()),
        }
    }
}
