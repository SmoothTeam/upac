// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs::{File, create_dir_all, read_link, symlink_metadata, write};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use composefs::fsverity::FsVerityHashValue;
use composefs::generic_tree::Stat;
use composefs::repository::{ImportContext, Repository};
use composefs::tree::FileSystem;

use upac_abi::FileDiffKind;
use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use upac_types::{FileEntry, FileEntryScope, TmpPath};

use crate::composefs::error::RepoError;
use crate::composefs::file::{FileHandle, stat_from_metadata};
use crate::composefs::repository::{ObjectID, commit_tree};
use crate::database::error::DeployRecordError;
use crate::database::files::FileStoreMut;
use crate::database::meta::MetaStore;
use crate::database::record::DeployRecord;
use crate::database::{InMemory, MemoryDatabase};
use crate::deploy::Deploy;
use crate::deploy::digest::current_prefix_digest;
use crate::errors::CommonError;
use crate::layout::database::DATABASE_PATH;
use crate::mutated::files::{
    CommitMessage, FilesError, NewPrefixDigest, RequestedFileKind, RequestedFilePackage, Subject,
};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{RollbackGuard, Stage};

pub struct TransactionStage;

impl Stage<FilesError> for TransactionStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), FilesError> {
        let files = context.take::<Vec<String>>().ok_or(CommonError::MissingResult)?;
        let file_kind = context.get::<RequestedFileKind>().ok_or(CommonError::MissingResult)?;
        let file_package = context
            .get::<RequestedFilePackage>()
            .ok_or(CommonError::MissingResult)?;
        let tmp_path = context.get::<TmpPath>().ok_or(CommonError::MissingResult)?;
        let deploy = context.get::<Deploy>().ok_or(CommonError::MissingResult)?;
        let subject = context.get::<Subject>().ok_or(CommonError::MissingResult)?;
        let message = context.get::<CommitMessage>().ok_or(CommonError::MissingResult)?;

        let current_prefix = current_prefix_digest()?;
        let repository = deploy.open_repository()?;
        let mut tree = deploy.open_tree(&current_prefix)?;

        let database_bytes = FileHandle::new(DATABASE_PATH).read_file(&repository, &tree)?;
        let mut database = MemoryDatabase::open_in_memory(database_bytes)?;

        let uuid = database
            .find_package_uuid(&file_package.name, &file_package.arch, file_package.arch_sub.as_deref())?
            .ok_or(FilesError::PackageNotFound)?;

        let mut import_ctx = ImportContext::default();

        match file_kind.0 {
            FileDiffKind::Removed => {
                for path in &files {
                    FileHandle::new(path).remove_in_tree(&mut tree)?;
                    database.remove_user_file(uuid, path)?;
                }
            }
            FileDiffKind::Added | FileDiffKind::Modified => {
                for path in &files {
                    self.add_file(path, &repository, &mut tree, &mut import_ctx)?;
                    database.insert_package_file(
                        uuid,
                        &FileEntry {
                            path: path.clone(),
                            is_user: true,
                            scope: FileEntryScope::Prefix,
                        },
                    )?;
                }
            }
        }

        let database_bytes = database.into_bytes()?;
        let database_scratch_path = format!("{}/files-packages.redb", tmp_path.as_ref());
        write(&database_scratch_path, &database_bytes).map_err(RepoError::from)?;

        FileHandle::new(DATABASE_PATH).insert_file(
            &repository,
            &mut tree,
            &File::open(&database_scratch_path).map_err(RepoError::from)?,
            Stat::uninitialized(),
            &mut import_ctx,
        )?;

        let digest = commit_tree(&repository, tree)?;
        let new_prefix = digest.to_hex();

        let current_record_dir = deploy.deploy(&current_prefix);
        let current_record = DeployRecord::read(&current_record_dir)?;

        let new_record_dir = deploy.deploy(&new_prefix);
        let mut written = Vec::new();
        if DeployRecord::read(&new_record_dir).is_err() {
            create_dir_all(&new_record_dir).map_err(DeployRecordError::from)?;

            let record = DeployRecord {
                prefix_digest: new_prefix.clone(),
                subject: subject.0.clone(),
                message: message.0.clone(),
                seq: DeployRecord::allocate_seq(deploy)?,
                timestamp: now_secs(),
                config_history: current_record.config_history.clone(),
                working_config: current_record.working_config.clone(),
            };
            written.push(record.write(&new_record_dir)?);
        }

        context.put(NewPrefixDigest(new_prefix));

        Ok((progress, Box::new(written)))
    }
}

impl TransactionStage {
    fn add_file(
        &self, path: &str, repository: &Repository<ObjectID>, tree: &mut FileSystem<ObjectID>,
        import_ctx: &mut ImportContext,
    ) -> Result<(), FilesError> {
        let source_path = Path::new(path);
        let metadata = symlink_metadata(source_path).map_err(RepoError::from)?;
        let stat = stat_from_metadata(&metadata);
        let handle = FileHandle::new(path);

        handle.remove_in_tree(tree)?;

        if metadata.is_symlink() {
            handle.symlink_in_tree(tree, read_link(source_path).map_err(RepoError::from)?, stat)?;
        } else {
            handle.insert_file(
                repository,
                tree,
                &File::open(source_path).map_err(RepoError::from)?,
                stat,
                import_ctx,
            )?;
        }

        Ok(())
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}
