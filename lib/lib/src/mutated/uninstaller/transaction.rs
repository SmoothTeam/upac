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

struct RemovePackageData<'a> {
    tree: &'a mut FileSystem<ObjectID>,
    database: &'a mut MemoryDatabase,
    removed_config_paths: &'a mut Vec<String>,
    purge: bool,
    cancel: &'a CancelToken,
    context: &'a Context,
    stage: u32,
}

impl Stage<UninstallError> for TransactionStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, mut progress: ProgressEventBuilder,
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

        let stage = progress.stage();
        let uuids_total = uuids.0.len() as u64;

        for (index, uuid) in uuids.0.iter().enumerate() {
            if cancel.is_cancelled() {
                return Err(CommonError::Cancelled.into());
            }

            let subject = database
                .get_package_meta(*uuid)?
                .map(|meta| meta.name)
                .unwrap_or_default();
            progress = progress.subject(subject).progress(index as u64, uuids_total);
            context.send_progress(&progress);

            let mut package_data = RemovePackageData {
                tree: &mut tree,
                database: &mut database,
                removed_config_paths: &mut removed_config_paths,
                purge: purge.0,
                cancel,
                context: &*context,
                stage,
            };

            self.remove_package(*uuid, &mut package_data)?;
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
    fn remove_package(&self, uuid: Uuid, package_data: &mut RemovePackageData) -> Result<(), UninstallError> {
        let files = package_data.database.list_package_files(uuid)?;
        let files_total = files.len() as u64;

        for (index, entry) in files.into_iter().enumerate() {
            if package_data.cancel.is_cancelled() {
                return Err(CommonError::Cancelled.into());
            }

            if entry.is_user && !package_data.purge {
                continue;
            }

            let event = ProgressEventBuilder::new(package_data.stage)
                .subject(entry.path.clone())
                .progress(index as u64, files_total);
            package_data.context.send_progress(&event);

            match entry.scope {
                FileEntryScope::Prefix => {
                    FileHandle::new(&entry.path).remove_in_tree(package_data.tree)?;
                }
                FileEntryScope::Config => {
                    package_data.removed_config_paths.push(entry.path.clone());
                }
            }

            if entry.is_user {
                package_data.database.remove_user_file(uuid, &entry.path)?;
            } else {
                package_data.database.remove_package_file(uuid, &entry.path)?;
            }
        }

        let meta = package_data
            .database
            .get_package_meta(uuid)?
            .ok_or(UninstallError::PackageNotFound)?;
        package_data
            .database
            .remove_package_meta(&meta.name, &meta.arch, meta.arch_sub.as_deref())?;

        Ok(())
    }
}
