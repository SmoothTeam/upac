// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs::{File, write};

use composefs::fsverity::FsVerityHashValue;
use composefs::generic_tree::Stat;
use composefs::repository::ImportContext;
use composefs::tree::FileSystem;

use uuid::Uuid;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use upac_types::{FileEntryScope, TmpPath};

use crate::composefs::error::RepoError;
use crate::composefs::file::FileHandle;
use crate::composefs::repository::{ObjectID, commit_tree};
use crate::database::files::{FileStore, FileStoreMut};
use crate::database::meta::{MetaStore, MetaStoreMut};
use crate::database::{InMemory, MemoryDatabase};
use crate::deploy::Deploy;
use crate::deploy::digest::current_prefix_digest;
use crate::errors::CommonError;
use crate::layout::database::{DATABASE_PATH, UNINSTALL_SCRATCH_FILENAME};
use crate::mutated::uninstaller::{NewPrefixDigest, PackageUuidsToRemove, Purge, RemovedConfigPaths, UninstallError};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage};

pub struct TransactionStage;

impl Stage<UninstallError> for TransactionStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), UninstallError> {
        let uuids = context
            .take::<PackageUuidsToRemove>()
            .ok_or(CommonError::MissingResult)?;
        let tmp_path = context.get::<TmpPath>().ok_or(CommonError::MissingResult)?;
        let deploy = context.get::<Deploy>().ok_or(CommonError::MissingResult)?;
        let purge = context.get::<Purge>().ok_or(CommonError::MissingResult)?;

        let current_prefix = current_prefix_digest()?;
        let repository = deploy.open_repository()?;
        let mut tree = deploy.open_tree(&current_prefix)?;

        let database_bytes = FileHandle::new(DATABASE_PATH).read_file(&repository, &tree)?;
        let mut database = MemoryDatabase::open_in_memory(database_bytes)?;

        let mut removed_config_paths = Vec::new();

        for uuid in &uuids.0 {
            if cancel.is_cancelled() {
                return Err(CommonError::Cancelled.into());
            }

            self.remove_package(
                *uuid,
                &mut tree,
                &mut database,
                &mut removed_config_paths,
                purge.0,
                cancel,
            )?;
        }

        let database_bytes = database.into_bytes()?;
        let database_scratch_path = format!("{}/{UNINSTALL_SCRATCH_FILENAME}", tmp_path.as_ref());
        write(&database_scratch_path, &database_bytes).map_err(RepoError::from)?;

        FileHandle::new(DATABASE_PATH).insert_file(
            &repository,
            &mut tree,
            &File::open(&database_scratch_path).map_err(RepoError::from)?,
            Stat::uninitialized(),
            &mut ImportContext::default(),
        )?;

        let digest = commit_tree(&repository, tree)?;

        context.put(NewPrefixDigest(digest.to_hex()));
        context.put(RemovedConfigPaths(removed_config_paths));

        Ok((progress, Box::new(NoRollback)))
    }
}

impl TransactionStage {
    fn remove_package(
        &self, uuid: Uuid, tree: &mut FileSystem<ObjectID>, database: &mut MemoryDatabase,
        removed_config_paths: &mut Vec<String>, purge: bool, cancel: &CancelToken,
    ) -> Result<(), UninstallError> {
        for entry in database.list_package_files(uuid)? {
            if cancel.is_cancelled() {
                return Err(CommonError::Cancelled.into());
            }

            if entry.is_user && !purge {
                continue;
            }

            match entry.scope {
                FileEntryScope::Prefix => {
                    FileHandle::new(&entry.path).remove_in_tree(tree)?;
                }
                FileEntryScope::Config => {
                    removed_config_paths.push(entry.path.clone());
                }
            }

            if entry.is_user {
                database.remove_user_file(uuid, &entry.path)?;
            } else {
                database.remove_package_file(uuid, &entry.path)?;
            }
        }

        let meta = database
            .get_package_meta(uuid)?
            .ok_or(UninstallError::PackageNotFound)?;
        database.remove_package_meta(&meta.name, &meta.arch, meta.arch_sub.as_deref())?;

        Ok(())
    }
}
