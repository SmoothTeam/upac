// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsStr;
use std::fs::{File, Permissions, create_dir_all, read_dir, read_link, set_permissions, write};
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use composefs::generic_tree::Stat;
use composefs::tree::{Inode, Leaf, LeafContent, RegularFile};

use upac_types::CancelToken;

use super::super::ObjectID;
use super::super::error::RepoError;
use super::{BTreeMap, Tree};

impl Tree {
    pub fn import_dir(
        &mut self, path: impl AsRef<Path>, source_dir: &Path, cancel: &CancelToken, on_entry: &mut dyn FnMut(&Path),
    ) -> Result<Vec<PathBuf>, RepoError> {
        let path = path.as_ref();
        let mut imported = Vec::new();

        for entry in read_dir(source_dir)? {
            if cancel.is_cancelled() {
                return Err(RepoError::Cancelled);
            }

            let entry = entry?;
            let source_path = entry.path();
            let metadata = entry.metadata()?;
            let stat = Stat {
                st_mode: metadata.mode(),
                st_uid: metadata.uid(),
                st_gid: metadata.gid(),
                st_mtim_sec: metadata.mtime(),
                st_mtim_nsec: metadata.mtime_nsec() as u32,
                xattrs: BTreeMap::new(),
            };
            let name = PathBuf::from(entry.file_name());
            let target = path.join(&name);

            if metadata.is_dir() {
                if self.set_dir_stat(&target, stat.clone()).is_err() {
                    self.remove(&target)?;
                    self.insert_dir(&target, stat)?;
                }

                let nested = self.import_dir(&target, &source_path, cancel, on_entry)?;
                imported.extend(nested.into_iter().map(|relative| name.join(relative)));
            } else if metadata.is_symlink() {
                self.insert_symlink(&target, read_link(&source_path)?, stat)?;
                on_entry(&name);
                imported.push(name);
            } else {
                self.insert_file(&target, &File::open(&source_path)?, stat)?;
                on_entry(&name);
                imported.push(name);
            }
        }

        Ok(imported)
    }

    pub fn export_dir(&self, path: impl AsRef<Path>, dest_dir: &Path, cancel: &CancelToken) -> Result<(), RepoError> {
        let path = path.as_ref();
        create_dir_all(dest_dir)?;

        for (name, inode) in self.entries(path)? {
            if cancel.is_cancelled() {
                return Err(RepoError::Cancelled);
            }

            let dest_path = dest_dir.join(name);

            match inode {
                Inode::Directory(directory) => {
                    self.export_dir(path.join(name), &dest_path, cancel)?;
                    set_permissions(&dest_path, Permissions::from_mode(directory.stat.st_mode))?;
                }
                Inode::Leaf(leaf_id, _) => self.export_leaf(self.filesystem.leaf(*leaf_id), &dest_path)?,
            }
        }

        Ok(())
    }

    fn export_leaf(&self, leaf: &Leaf<ObjectID>, dest_path: &Path) -> Result<(), RepoError> {
        match &leaf.content {
            LeafContent::Symlink(target) => Self::export_symlink(target, dest_path),
            LeafContent::Regular(regular) => self.export_regular_file(regular, leaf.stat.st_mode, dest_path),
            LeafContent::BlockDevice(_) | LeafContent::CharacterDevice(_) | LeafContent::Fifo | LeafContent::Socket => {
                Ok(())
            }
        }
    }

    fn export_symlink(target: &OsStr, dest_path: &Path) -> Result<(), RepoError> {
        symlink(target, dest_path)?;

        Ok(())
    }

    fn export_regular_file(
        &self, regular: &RegularFile<ObjectID>, mode: u32, dest_path: &Path,
    ) -> Result<(), RepoError> {
        write(dest_path, self.regular_content(regular)?)?;
        set_permissions(dest_path, Permissions::from_mode(mode))?;

        Ok(())
    }
}
