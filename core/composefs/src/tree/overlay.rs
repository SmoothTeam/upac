// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, Metadata, read_dir, read_link};
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};

use composefs::generic_tree::Stat;
use composefs::tree::Inode;

use super::super::error::RepoError;
use super::super::layout::deployment::OVERLAY_OPAQUE_XATTR;
use super::{BTreeMap, Tree};

impl Tree {
    pub fn apply_overlay_upper(&mut self, upper_dir: &Path) -> Result<(), RepoError> {
        self.apply_overlay_upper_at(&PathBuf::new(), upper_dir)
    }

    pub fn overlay(&mut self, other: &Tree) -> Result<(), RepoError> {
        self.overlay_at(&PathBuf::new(), other)
    }

    fn apply_overlay_upper_at(&mut self, path: &Path, upper_dir: &Path) -> Result<(), RepoError> {
        for entry in read_dir(upper_dir)? {
            let entry = entry?;
            let source_path = entry.path();
            let metadata = entry.metadata()?;
            let child = path.join(entry.file_name());

            if Self::is_whiteout(&metadata) {
                self.remove(&child)?;
                continue;
            }

            let stat = Stat {
                st_mode: metadata.mode(),
                st_uid: metadata.uid(),
                st_gid: metadata.gid(),
                st_mtim_sec: metadata.mtime(),
                st_mtim_nsec: metadata.mtime_nsec() as u32,
                xattrs: BTreeMap::new(),
            };

            if metadata.is_dir() {
                if Self::is_opaque(&source_path)? || self.stat(&child).is_err() {
                    self.remove(&child)?;
                    self.insert_dir(&child, stat)?;
                } else {
                    self.set_dir_stat(&child, stat)?;
                }

                self.apply_overlay_upper_at(&child, &source_path)?;
            } else if metadata.is_symlink() {
                self.remove(&child)?;
                self.insert_symlink(&child, read_link(&source_path)?, stat)?;
            } else {
                self.remove(&child)?;
                self.insert_file(&child, &File::open(&source_path)?, stat)?;
            }
        }

        Ok(())
    }

    fn overlay_at(&mut self, path: &Path, other: &Tree) -> Result<(), RepoError> {
        for (name, inode) in other.entries(path)? {
            let child = path.join(name);

            match inode {
                Inode::Directory(directory) => {
                    if self.stat(&child).is_err() {
                        self.insert_dir(&child, directory.stat.clone())?;
                    } else {
                        self.set_dir_stat(&child, directory.stat.clone())?;
                    }

                    self.overlay_at(&child, other)?;
                }
                Inode::Leaf(..) => {
                    self.remove(&child)?;
                    self.copy_leaf(&child, other, &child)?;
                }
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
}
