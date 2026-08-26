// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs::{File, copy, create_dir_all, read_link, remove_file, symlink_metadata, write};
use std::os::unix::fs::symlink;
use std::path::Path;

use composefs::fsverity::FsVerityHashValue;
use composefs::generic_tree::Stat;
use composefs::repository::{ImportContext, Repository};
use composefs::tree::FileSystem;

use uuid::Uuid;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};
use upac_abi::{DiffFileSource, FileDiffKind};

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
use crate::layout::database::{DATABASE_PATH, FILES_SCRATCH_FILENAME};
use crate::layout::deployment::{ETC_UPPER_RELATIVE_PATH, LIVE_ETC_DIR};
use crate::mutated::files::{
    CommitMessage, FilesError, NewPrefixDigest, RequestedFileKind, RequestedFilePackage, RequestedFileScope, Subject,
};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{RollbackGuard, Stage};

pub struct TransactionStage;

struct PrefixFilesData<'a> {
    repository: &'a Repository<ObjectID>,
    tree: &'a mut FileSystem<ObjectID>,
    database: &'a mut MemoryDatabase,
    import_ctx: &'a mut ImportContext,
    cancel: &'a CancelToken,
    context: &'a Context,
    stage: u32,
}

struct ConfigFilesData<'a> {
    etc_upper_dir: &'a Path,
    database: &'a mut MemoryDatabase,
    cancel: &'a CancelToken,
    context: &'a Context,
    stage: u32,
}

impl Stage<FilesError> for TransactionStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), FilesError> {
        let files = context.take::<Vec<String>>().ok_or(CommonError::MissingResult)?;
        let file_kind = context.get::<RequestedFileKind>().ok_or(CommonError::MissingResult)?;
        let scope = context.get::<RequestedFileScope>().ok_or(CommonError::MissingResult)?;
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

        let current_record_dir = deploy.deploy(&current_prefix);
        let etc_upper_dir = current_record_dir.join(ETC_UPPER_RELATIVE_PATH);

        let database_bytes = FileHandle::new(DATABASE_PATH).read_file(&repository, &tree)?;
        let mut database = MemoryDatabase::open_in_memory(database_bytes)?;

        let uuid = database
            .find_package_uuid(&file_package.name, &file_package.arch, file_package.arch_sub.as_deref())?
            .ok_or(FilesError::PackageNotFound)?;

        let mut import_ctx = ImportContext::default();
        let stage = progress.stage();

        match scope.0 {
            DiffFileSource::Prefix => {
                let mut data = PrefixFilesData {
                    repository: &repository,
                    tree: &mut tree,
                    database: &mut database,
                    import_ctx: &mut import_ctx,
                    cancel,
                    context: &*context,
                    stage,
                };
                self.apply_prefix_files(uuid, &files, file_kind.0, &mut data)?;
            }
            DiffFileSource::Config => {
                let mut data = ConfigFilesData {
                    etc_upper_dir: &etc_upper_dir,
                    database: &mut database,
                    cancel,
                    context: &*context,
                    stage,
                };
                self.apply_config_files(uuid, &files, file_kind.0, &mut data)?;
            }
        }

        let database_bytes = database.into_bytes()?;
        let database_scratch_path = format!("{}/{FILES_SCRATCH_FILENAME}", tmp_path.as_ref());
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

        let current_record = DeployRecord::read(&current_record_dir)?;

        let new_record_dir = deploy.deploy(&new_prefix);
        let mut written = Vec::new();
        if DeployRecord::read(&new_record_dir).is_err() {
            create_dir_all(&new_record_dir).map_err(DeployRecordError::from)?;

            let record = DeployRecord {
                prefix_digest: new_prefix.clone(),
                subject: subject.0.clone(),
                message: message.0.clone(),
                seq: DeployRecord::allocate_seq(&deploy.next_seq_path())?,
                timestamp: DeployRecord::now_secs(),
                config_history: current_record.config_history.clone(),
                working_config: current_record.working_config.clone(),
                pinned: false,
            };
            written.push(record.write(&new_record_dir)?);
        }

        context.put(NewPrefixDigest(new_prefix));

        Ok((progress, Box::new(written)))
    }
}

impl TransactionStage {
    fn apply_prefix_files(
        &self, uuid: Uuid, files: &[String], kind: FileDiffKind, data: &mut PrefixFilesData,
    ) -> Result<(), FilesError> {
        let files_total = files.len() as u64;

        for (index, path) in files.iter().enumerate() {
            if data.cancel.is_cancelled() {
                return Err(CommonError::Cancelled.into());
            }

            let event = ProgressEventBuilder::new(data.stage)
                .subject(path.clone())
                .progress(index as u64, files_total);
            data.context.send_progress(&event);

            match kind {
                FileDiffKind::Removed => {
                    FileHandle::new(path).remove_in_tree(data.tree)?;
                    data.database.remove_user_file(uuid, path)?;
                }
                FileDiffKind::Added | FileDiffKind::Modified => {
                    self.add_file(path, data.repository, data.tree, data.import_ctx)?;
                    data.database.insert_package_file(
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

        Ok(())
    }

    fn apply_config_files(
        &self, uuid: Uuid, files: &[String], kind: FileDiffKind, data: &mut ConfigFilesData,
    ) -> Result<(), FilesError> {
        let files_total = files.len() as u64;

        for (index, path) in files.iter().enumerate() {
            if data.cancel.is_cancelled() {
                return Err(CommonError::Cancelled.into());
            }

            let event = ProgressEventBuilder::new(data.stage)
                .subject(path.clone())
                .progress(index as u64, files_total);
            data.context.send_progress(&event);

            match kind {
                FileDiffKind::Removed => {
                    remove_file(data.etc_upper_dir.join(path)).map_err(RepoError::from)?;
                    data.database.remove_user_file(uuid, path)?;
                }
                FileDiffKind::Added | FileDiffKind::Modified => {
                    self.add_config_file(path, data.etc_upper_dir)?;
                    data.database.insert_package_file(
                        uuid,
                        &FileEntry {
                            path: path.clone(),
                            is_user: true,
                            scope: FileEntryScope::Config,
                        },
                    )?;
                }
            }
        }

        Ok(())
    }

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

    fn add_config_file(&self, path: &str, etc_upper_dir: &Path) -> Result<(), FilesError> {
        let live_path = Path::new(LIVE_ETC_DIR).join(path);
        let metadata = symlink_metadata(&live_path).map_err(RepoError::from)?;
        let dest_path = etc_upper_dir.join(path);

        if let Some(parent) = dest_path.parent() {
            create_dir_all(parent).map_err(RepoError::from)?;
        }

        if metadata.is_symlink() {
            let _ = remove_file(&dest_path);
            symlink(read_link(&live_path).map_err(RepoError::from)?, &dest_path).map_err(RepoError::from)?;
        } else {
            copy(&live_path, &dest_path).map_err(RepoError::from)?;
        }

        Ok(())
    }
}
