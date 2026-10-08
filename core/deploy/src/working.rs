// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::{Path, PathBuf};

use composefs::generic_tree::Stat;

use uuid::Uuid;

use upac_types::CancelToken;
use upac_types::decoder::PackageTriggers;
use upac_types::diff::DiffFileSource;
use upac_types::package::PackageMeta;
use upac_types::response::entry::{FileEntry, FileEntryScope};
use upac_types::transaction::{Transaction, TransactionKind};

use upac_composefs::tree::Tree;
use upac_composefs::{Digest, Repo};

use upac_database::MemoryDatabase;
use upac_database::files::{FileStore, FileStoreMut};
use upac_database::layout::database::DATABASE_PATH;
use upac_database::meta::{MetaStore, MetaStoreMut};
use upac_database::transaction::TransactionStoreMut;
use upac_database::triggers::TriggerStoreMut;

use super::error::PrefixEditError;
use super::layout::prefix::{DEFAULTS_DIR, SYSTEM_CONFIG_DIR, SYSTEM_PREFIX_DIR};

pub struct CommittedPrefix {
    pub digest: Digest,
    pub transaction: Transaction,
    pub defaults: Tree,
}

pub struct WorkingPrefix {
    repo: Repo,
    tree: Tree,
    database: MemoryDatabase,
    parent: Option<String>,
}

impl WorkingPrefix {
    pub(crate) fn new(repo: Repo, tree: Tree, database: MemoryDatabase, parent: Option<String>) -> Self {
        Self {
            repo,
            tree,
            database,
            parent,
        }
    }

    pub fn database(&self) -> &MemoryDatabase {
        &self.database
    }

    pub fn add_package(
        &mut self, meta: &PackageMeta, triggers: &PackageTriggers, unpacked_root: &Path, cancel: &CancelToken,
    ) -> Result<Uuid, PrefixEditError> {
        if self
            .database
            .find_package_uuid(&meta.name, &meta.arch, meta.arch_sub.as_deref())?
            .is_some()
        {
            return Err(PrefixEditError::PackageExists);
        }

        let files = self.import_package_files(unpacked_root, cancel)?;

        let uuid = self.database.insert_package_meta(meta)?;
        self.database.set_package_triggers(uuid, triggers)?;
        self.record_package_files(uuid, files)?;

        Ok(uuid)
    }

    pub fn replace_package(
        &mut self, uuid: Uuid, meta: &PackageMeta, triggers: &PackageTriggers, unpacked_root: &Path,
        cancel: &CancelToken,
    ) -> Result<(), PrefixEditError> {
        for entry in self.database.list_package_files(uuid)? {
            self.tree.remove(Self::tree_path(&entry))?;
            self.database.remove_package_file(uuid, &entry.path)?;
        }

        let files = self.import_package_files(unpacked_root, cancel)?;

        self.database.update_package_meta(meta)?;
        self.database.set_package_triggers(uuid, triggers)?;
        self.record_package_files(uuid, files)
    }

    pub fn remove_package(&mut self, uuid: Uuid, purge_user_files: bool) -> Result<(), PrefixEditError> {
        for entry in self.database.list_package_files(uuid)? {
            if entry.is_user && !purge_user_files {
                continue;
            }

            self.tree.remove(Self::tree_path(&entry))?;

            if entry.is_user {
                self.database.remove_user_file(uuid, &entry.path)?;
            } else {
                self.database.remove_package_file(uuid, &entry.path)?;
            }
        }

        let meta = self
            .database
            .get_package_meta(uuid)?
            .ok_or(PrefixEditError::PackageNotFound)?;
        self.database
            .remove_package_meta(&meta.name, &meta.arch, meta.arch_sub.as_deref())?;
        self.database.remove_package_triggers(uuid)?;

        Ok(())
    }

    pub fn attach_file(
        &mut self, owner: Uuid, scope: DiffFileSource, system_path: &str, source: &Path,
    ) -> Result<(), PrefixEditError> {
        let entry = Self::user_entry(scope, system_path)?;
        let tree_path = Self::tree_path(&entry);

        self.insert_missing_parents(&tree_path)?;
        self.tree.import_path(&tree_path, source)?;
        self.database.insert_package_file(owner, &entry)?;

        Ok(())
    }

    pub fn detach_file(
        &mut self, owner: Uuid, scope: DiffFileSource, system_path: &str,
    ) -> Result<(), PrefixEditError> {
        let entry = Self::user_entry(scope, system_path)?;

        self.tree.remove(Self::tree_path(&entry))?;
        self.database.remove_user_file(owner, &entry.path)?;

        Ok(())
    }

    pub fn add_unowned_dir(
        &mut self, source_dir: &Path, cancel: &CancelToken, on_entry: &mut dyn FnMut(&Path),
    ) -> Result<(), PrefixEditError> {
        if source_dir.is_dir() {
            self.tree.import_dir("", source_dir, cancel, on_entry)?;
        }

        Ok(())
    }

    pub fn commit(
        mut self, kind: TransactionKind, subject: String, message: Option<String>,
    ) -> Result<CommittedPrefix, PrefixEditError> {
        let transaction = Transaction::new(self.parent.take(), kind, subject, message);
        self.database.set_transaction(&transaction)?;

        self.insert_missing_parents(Path::new(DATABASE_PATH))?;

        let database_bytes = self.database.into_bytes()?;
        self.tree
            .insert_bytes(DATABASE_PATH, &database_bytes, Stat::uninitialized())?;

        let defaults = if self.tree.contains(DEFAULTS_DIR) {
            self.tree.copy_tree(DEFAULTS_DIR)?
        } else {
            self.repo.empty_tree()
        };

        Ok(CommittedPrefix {
            digest: self.tree.commit()?,
            transaction,
            defaults,
        })
    }

    fn import_package_files(
        &mut self, unpacked_root: &Path, cancel: &CancelToken,
    ) -> Result<Vec<(PathBuf, FileEntryScope)>, PrefixEditError> {
        let mut files = Vec::new();

        let prefix_source = unpacked_root.join(SYSTEM_PREFIX_DIR);
        if prefix_source.is_dir() {
            let imported = self.tree.import_dir("", &prefix_source, cancel, &mut |_| {})?;
            files.extend(imported.into_iter().map(|path| (path, FileEntryScope::Prefix)));
        }

        let config_source = unpacked_root.join(SYSTEM_CONFIG_DIR);
        if config_source.is_dir() {
            if !self.tree.contains(DEFAULTS_DIR) {
                self.tree.insert_dir(DEFAULTS_DIR, Stat::uninitialized())?;
            }

            let imported = self
                .tree
                .import_dir(DEFAULTS_DIR, &config_source, cancel, &mut |_| {})?;
            files.extend(imported.into_iter().map(|path| (path, FileEntryScope::Config)));
        }

        Ok(files)
    }

    fn record_package_files(
        &mut self, uuid: Uuid, files: Vec<(PathBuf, FileEntryScope)>,
    ) -> Result<(), PrefixEditError> {
        for (path, scope) in files {
            self.database.insert_package_file(
                uuid,
                &FileEntry {
                    path: path.to_string_lossy().into_owned(),
                    is_user: false,
                    scope,
                },
            )?;
        }

        Ok(())
    }

    fn insert_missing_parents(&mut self, path: &Path) -> Result<(), PrefixEditError> {
        let mut missing_parents: Vec<&Path> = path
            .ancestors()
            .skip(1)
            .filter(|ancestor| !ancestor.as_os_str().is_empty() && !self.tree.contains(ancestor))
            .collect();
        missing_parents.reverse();

        for parent in missing_parents {
            self.tree.insert_dir(parent, Stat::uninitialized())?;
        }

        Ok(())
    }

    fn user_entry(scope: DiffFileSource, system_path: &str) -> Result<FileEntry, PrefixEditError> {
        let (system_dir, entry_scope) = match scope {
            DiffFileSource::Prefix => (SYSTEM_PREFIX_DIR, FileEntryScope::Prefix),
            DiffFileSource::Config => (SYSTEM_CONFIG_DIR, FileEntryScope::Config),
        };

        let relative = Path::new(system_path)
            .strip_prefix(Path::new("/").join(system_dir))
            .map_err(|_| PrefixEditError::OutsideSystemDir)?;
        if relative.as_os_str().is_empty() {
            return Err(PrefixEditError::OutsideSystemDir);
        }

        Ok(FileEntry {
            path: relative.to_string_lossy().into_owned(),
            is_user: true,
            scope: entry_scope,
        })
    }

    fn tree_path(entry: &FileEntry) -> PathBuf {
        match entry.scope {
            FileEntryScope::Prefix => PathBuf::from(&entry.path),
            FileEntryScope::Config => Path::new(DEFAULTS_DIR).join(&entry.path),
        }
    }
}
