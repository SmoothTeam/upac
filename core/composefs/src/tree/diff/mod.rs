// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::BTreeMap;
use std::path::PathBuf;

use composefs::tree::{Inode, Leaf, LeafContent, RegularFile};

use upac_types::diff::FileDiffKind;

use super::Tree;

use crate::ObjectID;

pub struct Diff<'tree> {
    from: &'tree Tree,
    to: &'tree Tree,

    changes: BTreeMap<PathBuf, FileDiffKind>,
}

impl<'tree> Diff<'tree> {
    pub fn new(from: &'tree Tree, to: &'tree Tree) -> Self {
        Self {
            from,
            to,
            changes: BTreeMap::new(),
        }
    }

    pub fn diff(&mut self) -> &BTreeMap<PathBuf, FileDiffKind> {
        fn content_matches(from: &Leaf<ObjectID>, to: &Leaf<ObjectID>) -> bool {
            match (&from.content, &to.content) {
                (
                    LeafContent::Regular(RegularFile::External(from_id, from_size)),
                    LeafContent::Regular(RegularFile::External(to_id, to_size)),
                ) => from_id == to_id && from_size == to_size,
                (
                    LeafContent::Regular(RegularFile::Inline(from_bytes)),
                    LeafContent::Regular(RegularFile::Inline(to_bytes)),
                ) => from_bytes == to_bytes,
                (LeafContent::Symlink(from_target), LeafContent::Symlink(to_target)) => from_target == to_target,
                _ => false,
            }
        }

        fn leaves(tree: &Tree) -> BTreeMap<PathBuf, &Leaf<ObjectID>> {
            let view = tree.view();

            view.into_iter()
                .filter_map(|(path, inode)| match inode {
                    Inode::Leaf(leaf_id, _) => Some((path, view.filesystem().leaf(*leaf_id))),
                    Inode::Directory(_) => None,
                })
                .filter(|(_, leaf)| matches!(leaf.content, LeafContent::Regular(_) | LeafContent::Symlink(_)))
                .collect()
        }

        let mut to = leaves(self.to);

        for (path, from_leaf) in leaves(self.from) {
            let kind = match to.remove(&path) {
                None => FileDiffKind::Removed,
                Some(to_leaf) if content_matches(from_leaf, to_leaf) => continue,
                Some(_) => FileDiffKind::Modified,
            };

            self.changes.insert(path, kind);
        }

        self.changes
            .extend(to.into_keys().map(|path| (path, FileDiffKind::Added)));

        &self.changes
    }
}
