// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::cmp::Ordering;
use std::path::{Path, PathBuf};

use composefs::tree::{Directory, FileSystem, Inode, Leaf, LeafContent, RegularFile};

use upac_abi::FileDiffKind;

use crate::composefs::repository::ObjectID;

macro_rules! advance_subtree {
    ($differ:expr, $prefix:expr, $entries:expr, $next:expr, $name:expr, $inode:expr, $side:expr) => {{
        $differ.mark_subtree(&$prefix.join($name), $inode, $side);
        $next = $entries.next();
    }};
}

#[derive(Clone, Copy)]
enum Side {
    From,
    To,
}

impl Side {
    fn kind(self) -> FileDiffKind {
        match self {
            Side::From => FileDiffKind::Removed,
            Side::To => FileDiffKind::Added,
        }
    }
}

pub struct TreeDiff<'a> {
    from_leaves: &'a [Leaf<ObjectID>],
    to_leaves: &'a [Leaf<ObjectID>],
    changes: Vec<(String, FileDiffKind)>,
}

impl<'a> TreeDiff<'a> {
    pub fn run(from: &'a FileSystem<ObjectID>, to: &'a FileSystem<ObjectID>) -> Vec<(String, FileDiffKind)> {
        let mut differ = Self {
            from_leaves: &from.leaves,
            to_leaves: &to.leaves,
            changes: Vec::new(),
        };

        differ.compare_directories(&PathBuf::new(), &from.root, &to.root);

        differ.changes
    }

    fn compare_directories(&mut self, prefix: &Path, from_dir: &Directory<ObjectID>, to_dir: &Directory<ObjectID>) {
        let mut from_entries = from_dir.sorted_entries();
        let mut to_entries = to_dir.sorted_entries();

        let mut from_next = from_entries.next();
        let mut to_next = to_entries.next();

        loop {
            match (from_next, to_next) {
                (Some((from_name, from_inode)), Some((to_name, to_inode))) => match from_name.cmp(to_name) {
                    Ordering::Less => {
                        advance_subtree!(self, prefix, from_entries, from_next, from_name, from_inode, Side::From)
                    }
                    Ordering::Greater => {
                        advance_subtree!(self, prefix, to_entries, to_next, to_name, to_inode, Side::To)
                    }
                    Ordering::Equal => {
                        self.compare_matched_entry(&prefix.join(from_name), from_inode, to_inode);
                        from_next = from_entries.next();
                        to_next = to_entries.next();
                    }
                },
                (Some((from_name, from_inode)), None) => {
                    advance_subtree!(self, prefix, from_entries, from_next, from_name, from_inode, Side::From)
                }
                (None, Some((to_name, to_inode))) => {
                    advance_subtree!(self, prefix, to_entries, to_next, to_name, to_inode, Side::To)
                }
                (None, None) => break,
            }
        }
    }

    fn compare_matched_entry(&mut self, path: &Path, from_inode: &Inode<ObjectID>, to_inode: &Inode<ObjectID>) {
        match (from_inode, to_inode) {
            (Inode::Directory(from_sub), Inode::Directory(to_sub)) => {
                self.compare_directories(path, from_sub, to_sub);
            }
            (Inode::Leaf(from_id, _), Inode::Leaf(to_id, _)) => {
                let from_leaf = &self.from_leaves[from_id.0];
                let to_leaf = &self.to_leaves[to_id.0];

                if Self::is_regular_or_symlink(from_leaf)
                    && Self::is_regular_or_symlink(to_leaf)
                    && !Self::content_matches(from_leaf, to_leaf)
                {
                    self.changes.push((Self::path_to_string(path), FileDiffKind::Modified));
                }
            }
            (from_inode, to_inode) => {
                self.mark_subtree(path, from_inode, Side::From);
                self.mark_subtree(path, to_inode, Side::To);
            }
        }
    }

    fn mark_subtree(&mut self, path: &Path, inode: &Inode<ObjectID>, side: Side) {
        match inode {
            Inode::Leaf(id, _) => {
                let leaf = &self.leaves(side)[id.0];

                if Self::is_regular_or_symlink(leaf) {
                    self.changes.push((Self::path_to_string(path), side.kind()));
                }
            }
            Inode::Directory(dir) => {
                for (name, child) in dir.sorted_entries() {
                    self.mark_subtree(&path.join(name), child, side);
                }
            }
        }
    }

    fn leaves(&self, side: Side) -> &'a [Leaf<ObjectID>] {
        match side {
            Side::From => self.from_leaves,
            Side::To => self.to_leaves,
        }
    }

    fn is_regular_or_symlink(leaf: &Leaf<ObjectID>) -> bool {
        matches!(leaf.content, LeafContent::Regular(_) | LeafContent::Symlink(_))
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

    fn path_to_string(path: &Path) -> String {
        path.to_string_lossy().into_owned()
    }
}
