// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::iter::from_fn;
use std::path::PathBuf;

use composefs::tree::{Inode, Leaf, LeafContent, RegularFile};

use upac_types::diff::FileDiffKind;

use super::Tree;

use crate::ObjectID;

impl Tree {
    pub fn diff(&self, other: &Tree) -> BTreeMap<PathBuf, FileDiffKind> {
        let mut changes = BTreeMap::new();
        let mut from = self.leaves().peekable();
        let mut to = other.leaves().peekable();

        loop {
            let side = match (from.peek(), to.peek()) {
                (None, None) => break,
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (Some((from_path, _)), Some((to_path, _))) => from_path.cmp(to_path),
            };

            match side {
                Ordering::Less => changes.extend(from.next().map(|(path, _)| (path, FileDiffKind::Removed))),
                Ordering::Greater => changes.extend(to.next().map(|(path, _)| (path, FileDiffKind::Added))),
                Ordering::Equal => {
                    if let (Some((path, from_leaf)), Some((_, to_leaf))) = (from.next(), to.next())
                        && !Self::content_matches(from_leaf, to_leaf)
                    {
                        changes.insert(path, FileDiffKind::Modified);
                    }
                }
            }
        }

        changes
    }

    pub fn leaves(&self) -> impl Iterator<Item = (PathBuf, &Leaf<ObjectID>)> {
        let mut pending = vec![(PathBuf::new(), self.filesystem.root.sorted_entries())];

        from_fn(move || {
            loop {
                let (path, entries) = pending.last_mut()?;
                let Some((name, inode)) = entries.next() else {
                    pending.pop();
                    continue;
                };
                let child = path.join(name);

                match inode {
                    Inode::Directory(directory) => pending.push((child, directory.sorted_entries())),
                    Inode::Leaf(leaf_id, _) => {
                        let leaf = self.filesystem.leaf(*leaf_id);
                        if matches!(leaf.content, LeafContent::Regular(_) | LeafContent::Symlink(_)) {
                            return Some((child, leaf));
                        }
                    }
                }
            }
        })
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
