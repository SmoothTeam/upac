// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use composefs::tree::{Directory, Inode, Leaf, LeafContent, RegularFile};

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
        let (mut from, mut to) = (BTreeMap::new(), BTreeMap::new());

        self.collect(self.from, &self.from.filesystem.root, Path::new(""), &mut from);
        self.collect(self.to, &self.to.filesystem.root, Path::new(""), &mut to);

        let (mut from, mut to) = (from.into_iter().peekable(), to.into_iter().peekable());

        loop {
            let side = match (from.peek(), to.peek()) {
                (None, None) => break,
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (Some((from_path, _)), Some((to_path, _))) => from_path.cmp(to_path),
            };

            match side {
                Ordering::Less => self
                    .changes
                    .extend(from.next().map(|(path, _)| (path, FileDiffKind::Removed))),
                Ordering::Greater => self
                    .changes
                    .extend(to.next().map(|(path, _)| (path, FileDiffKind::Added))),
                Ordering::Equal => {
                    if let (Some((path, from_leaf)), Some((_, to_leaf))) = (from.next(), to.next())
                        && !Self::content_matches(from_leaf, to_leaf)
                    {
                        self.changes.insert(path, FileDiffKind::Modified);
                    }
                }
            }
        }

        &self.changes
    }

    fn collect(
        &self, tree: &'tree Tree, directory: &'tree Directory<ObjectID>, path: &Path,
        leaves: &mut BTreeMap<PathBuf, &'tree Leaf<ObjectID>>,
    ) {
        for (name, inode) in directory.entries() {
            let child = path.join(name);

            match inode {
                Inode::Directory(subdirectory) => self.collect(tree, subdirectory, &child, leaves),
                Inode::Leaf(leaf_id, _) => {
                    let leaf = tree.filesystem.leaf(*leaf_id);
                    if matches!(leaf.content, LeafContent::Regular(_) | LeafContent::Symlink(_)) {
                        leaves.insert(child, leaf);
                    }
                }
            }
        }
    }

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
}
