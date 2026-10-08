// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsStr;
use std::fs::{
    File, Metadata, Permissions, create_dir_all, read_dir, read_link, set_permissions, symlink_metadata, write,
};
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
            let name = PathBuf::from(entry.file_name());
            let target = path.join(&name);
            on_entry(&target);

            let source_path = entry.path();
            let metadata = entry.metadata()?;

            if metadata.is_dir() {
                self.import_directory(&target, &metadata)?;
                let nested = self.import_dir(&target, &source_path, cancel, on_entry)?;
                imported.extend(nested.into_iter().map(|relative| name.join(relative)));
            } else if self.import_leaf(&target, &source_path, &metadata)? {
                imported.push(name);
            }
        }

        Ok(imported)
    }

    pub fn import_path(&mut self, path: impl AsRef<Path>, source: &Path) -> Result<(), RepoError> {
        let path = path.as_ref();
        let metadata = symlink_metadata(source)?;

        if metadata.is_dir() {
            return self.import_directory(path, &metadata);
        }

        self.remove(path)?;
        self.import_leaf(path, source, &metadata)?;

        Ok(())
    }

    fn import_directory(&mut self, target: &Path, metadata: &Metadata) -> Result<(), RepoError> {
        let stat = Self::file_stat(metadata);

        if self.set_dir_stat(target, stat.clone()).is_err() {
            self.remove(target)?;
            self.insert_dir(target, stat)?;
        }

        Ok(())
    }

    fn import_leaf(&mut self, target: &Path, source: &Path, metadata: &Metadata) -> Result<bool, RepoError> {
        let file_type = metadata.file_type();

        if file_type.is_symlink() {
            self.import_symlink(target, source, metadata)?;
        } else if file_type.is_file() {
            self.import_regular_file(target, source, metadata)?;
        } else {
            return Ok(false);
        }

        Ok(true)
    }

    fn import_symlink(&mut self, target: &Path, source: &Path, metadata: &Metadata) -> Result<(), RepoError> {
        self.insert_symlink(target, read_link(source)?, Self::file_stat(metadata))
    }

    fn import_regular_file(&mut self, target: &Path, source: &Path, metadata: &Metadata) -> Result<(), RepoError> {
        self.insert_file(target, &File::open(source)?, Self::file_stat(metadata))
    }

    fn file_stat(metadata: &Metadata) -> Stat {
        Stat {
            st_mode: metadata.mode(),
            st_uid: metadata.uid(),
            st_gid: metadata.gid(),
            st_mtim_sec: metadata.mtime(),
            st_mtim_nsec: metadata.mtime_nsec() as u32,
            xattrs: BTreeMap::new(),
        }
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
