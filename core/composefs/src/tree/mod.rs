// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{File, read_link, symlink_metadata};
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use composefs::MAX_INLINE_CONTENT;
use composefs::generic_tree::Stat;
use composefs::repository::ImportContext;
use composefs::tree::{Directory, FileSystem, Inode, LeafContent, RegularFile};

use super::error::RepoError;
use super::{Digest, ObjectID, Repo};

mod diff;
mod import;
mod merge;
mod overlay;

#[cfg(test)]
#[path = "../../tests/inline/tree.rs"]
mod tests;

pub struct MergeResult {
    pub tree: Tree,
    pub conflicts: Vec<PathBuf>,
}

pub struct Tree {
    repo: Repo,
    filesystem: FileSystem<ObjectID>,
    import_context: ImportContext,
}

impl Clone for Tree {
    fn clone(&self) -> Self {
        Self::new(self.repo.clone(), self.filesystem.clone())
    }
}

impl Tree {
    pub(crate) fn new(repo: Repo, filesystem: FileSystem<ObjectID>) -> Self {
        Self {
            repo,
            filesystem,
            import_context: ImportContext::default(),
        }
    }

    pub fn upstream(&self) -> &FileSystem<ObjectID> {
        &self.filesystem
    }

    pub fn commit(self) -> Result<Digest, RepoError> {
        let Tree {
            repo, mut filesystem, ..
        } = self;
        filesystem.compact();

        repo.commit_filesystem(filesystem)
    }

    pub fn contains(&self, path: impl AsRef<Path>) -> bool {
        self.inode(path.as_ref()).is_ok()
    }

    pub fn stat(&self, path: impl AsRef<Path>) -> Result<&Stat, RepoError> {
        Ok(self.inode(path.as_ref())?.stat(&self.filesystem.leaves))
    }

    pub fn read_file(&self, path: impl AsRef<Path>) -> Result<Vec<u8>, RepoError> {
        let (parent, filename) = self.filesystem.root.split(path.as_ref().as_os_str())?;
        let regular = parent.get_file(filename, &self.filesystem.leaves)?;

        self.regular_content(regular)
    }

    pub fn insert_dir(&mut self, path: impl AsRef<Path>, stat: Stat) -> Result<(), RepoError> {
        let (parent, filename) = self.filesystem.root.split_mut(path.as_ref().as_os_str())?;
        parent.insert(filename, Inode::Directory(Box::new(Directory::new(stat))));

        Ok(())
    }

    pub fn insert_file(&mut self, path: impl AsRef<Path>, source: &File, stat: Stat) -> Result<(), RepoError> {
        let size = source.metadata()?.len();

        let regular = if size <= MAX_INLINE_CONTENT as u64 {
            let mut content = Vec::with_capacity(size as usize);
            let mut reader = source;
            reader.read_to_end(&mut content)?;

            RegularFile::Inline(content.into())
        } else {
            let (object_id, _method) =
                self.repo
                    .upstream()
                    .ensure_object_from_file(source, size, &mut self.import_context)?;

            RegularFile::External(object_id, size)
        };

        self.insert_leaf(path.as_ref(), stat, LeafContent::Regular(regular))
    }

    pub fn insert_bytes(&mut self, path: impl AsRef<Path>, content: &[u8], stat: Stat) -> Result<(), RepoError> {
        let regular = if content.len() <= MAX_INLINE_CONTENT {
            RegularFile::Inline(content.into())
        } else {
            RegularFile::External(self.repo.upstream().ensure_object(content)?, content.len() as u64)
        };

        self.insert_leaf(path.as_ref(), stat, LeafContent::Regular(regular))
    }

    pub fn insert_symlink(
        &mut self, path: impl AsRef<Path>, target: impl AsRef<OsStr>, stat: Stat,
    ) -> Result<(), RepoError> {
        self.insert_leaf(path.as_ref(), stat, LeafContent::Symlink(target.as_ref().into()))
    }

    pub fn import_path(&mut self, path: impl AsRef<Path>, source: &Path) -> Result<(), RepoError> {
        let path = path.as_ref();
        let metadata = symlink_metadata(source)?;
        let stat = Stat {
            st_mode: metadata.mode(),
            st_uid: metadata.uid(),
            st_gid: metadata.gid(),
            st_mtim_sec: metadata.mtime(),
            st_mtim_nsec: metadata.mtime_nsec() as u32,
            xattrs: BTreeMap::new(),
        };

        if metadata.is_dir() {
            if self.set_dir_stat(path, stat.clone()).is_err() {
                self.remove(path)?;
                self.insert_dir(path, stat)?;
            }
        } else if metadata.is_symlink() {
            self.remove(path)?;
            self.insert_symlink(path, read_link(source)?, stat)?;
        } else {
            self.remove(path)?;
            self.insert_file(path, &File::open(source)?, stat)?;
        }

        Ok(())
    }

    pub fn copy_tree(&self, path: impl AsRef<Path>) -> Result<Tree, RepoError> {
        let path = path.as_ref();
        let source_dir = self.filesystem.root.get_directory(path.as_os_str())?;

        let mut copy = Tree::new(self.repo.clone(), FileSystem::new(source_dir.stat.clone()));
        copy.copy_entries(Path::new(""), self, path)?;

        Ok(copy)
    }

    pub fn remove(&mut self, path: impl AsRef<Path>) -> Result<(), RepoError> {
        let (parent, filename) = self.filesystem.root.split_mut(path.as_ref().as_os_str())?;
        parent.remove(filename);

        Ok(())
    }

    fn inode(&self, path: &Path) -> Result<&Inode<ObjectID>, RepoError> {
        let (parent, filename) = self.filesystem.root.split(path.as_os_str())?;

        parent.lookup(filename).ok_or(RepoError::NotFound)
    }

    fn entries(&self, path: &Path) -> Result<impl Iterator<Item = (&OsStr, &Inode<ObjectID>)>, RepoError> {
        Ok(self.filesystem.root.get_directory(path.as_os_str())?.sorted_entries())
    }

    fn set_dir_stat(&mut self, path: &Path, stat: Stat) -> Result<(), RepoError> {
        self.filesystem.root.get_directory_mut(path.as_os_str())?.stat = stat;

        Ok(())
    }

    fn insert_leaf(&mut self, path: &Path, stat: Stat, content: LeafContent<ObjectID>) -> Result<(), RepoError> {
        let leaf_id = self.filesystem.push_leaf(stat, content);

        let (parent, filename) = self.filesystem.root.split_mut(path.as_os_str())?;
        parent.insert(filename, Inode::leaf(leaf_id));

        Ok(())
    }

    fn copy_entries(&mut self, path: &Path, source: &Tree, source_path: &Path) -> Result<(), RepoError> {
        for (name, inode) in source.entries(source_path)? {
            let child_path = path.join(name);
            let source_child_path = source_path.join(name);

            match inode {
                Inode::Directory(directory) => {
                    self.insert_dir(&child_path, directory.stat.clone())?;
                    self.copy_entries(&child_path, source, &source_child_path)?;
                }
                Inode::Leaf(..) => self.copy_leaf(&child_path, source, &source_child_path)?,
            }
        }

        Ok(())
    }

    fn copy_leaf(&mut self, path: &Path, source: &Tree, source_path: &Path) -> Result<(), RepoError> {
        let (source_parent, source_filename) = source.filesystem.root.split(source_path.as_os_str())?;
        let leaf_id = source_parent.leaf_id(source_filename)?;
        let leaf = source.filesystem.leaf(leaf_id).clone();

        self.insert_leaf(path, leaf.stat, leaf.content)
    }

    fn regular_content(&self, regular: &RegularFile<ObjectID>) -> Result<Vec<u8>, RepoError> {
        match regular {
            RegularFile::Inline(content) => Ok(content.to_vec()),
            RegularFile::External(object_id, _) | RegularFile::ExternalNoVerity(object_id, _) => {
                Ok(self.repo.upstream().read_object(object_id)?)
            }
            RegularFile::Sparse(size) => Ok(vec![0u8; *size as usize]),
        }
    }
}
