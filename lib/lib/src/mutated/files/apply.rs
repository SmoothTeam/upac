// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, copy, create_dir_all, read_link, remove_file, symlink_metadata};
use std::os::unix::fs::symlink;
use std::path::Path;

use composefs::repository::{ImportContext, Repository};
use composefs::tree::FileSystem;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};
use upac_abi::{DiffFileSource, FileDiffKind};

use upac_types::{FileEntry, FileEntryScope};

use crate::composefs::error::RepoError;
use crate::composefs::file::{FileHandle, stat_from_metadata};
use crate::composefs::repository::ObjectID;
use crate::database::files::FileStoreMut;
use crate::deploy::Deploy;
use crate::errors::CommonError;
use crate::layout::deployment::LIVE_ETC_DIR;
use crate::mutated::files::{
    EtcUpperDir, FilesError, PendingFiles, RequestedFileKind, RequestedFileScope, TargetUuid, TotalFiles,
    WorkingDatabase, WorkingTree,
};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

pub struct ApplyFileStage;

impl Stage<FilesError> for ApplyFileStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, mut progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), FilesError> {
        let mut pending = context.take::<PendingFiles>().ok_or(CommonError::MissingResult)?;
        let mut tree = context.take::<WorkingTree>().ok_or(CommonError::MissingResult)?;
        let mut database = context.take::<WorkingDatabase>().ok_or(CommonError::MissingResult)?;
        let mut import_ctx = context.take::<ImportContext>().ok_or(CommonError::MissingResult)?;
        let etc_upper_dir = context.get::<EtcUpperDir>().ok_or(CommonError::MissingResult)?;
        let uuid = context.get::<TargetUuid>().ok_or(CommonError::MissingResult)?;
        let file_kind = context.get::<RequestedFileKind>().ok_or(CommonError::MissingResult)?;
        let scope = context.get::<RequestedFileScope>().ok_or(CommonError::MissingResult)?;
        let total = context.get::<TotalFiles>().ok_or(CommonError::MissingResult)?;
        let deploy = context.get::<Deploy>().ok_or(CommonError::MissingResult)?;

        let path = pending.0.pop_front().ok_or(CommonError::MissingResult)?;

        match scope.0 {
            DiffFileSource::Prefix => {
                let repository = deploy.open_repository()?;

                match file_kind.0 {
                    FileDiffKind::Removed => {
                        FileHandle::new(&path).remove_in_tree(&mut tree.0)?;
                        database.0.remove_user_file(uuid.0, &path)?;
                    }
                    FileDiffKind::Added | FileDiffKind::Modified => {
                        Self::add_file(&path, &repository, &mut tree.0, &mut import_ctx)?;
                        database.0.insert_package_file(
                            uuid.0,
                            &FileEntry {
                                path: path.clone(),
                                is_user: true,
                                scope: FileEntryScope::Prefix,
                            },
                        )?;
                    }
                }
            }
            DiffFileSource::Config => match file_kind.0 {
                FileDiffKind::Removed => {
                    remove_file(etc_upper_dir.0.join(&path)).map_err(RepoError::from)?;
                    database.0.remove_user_file(uuid.0, &path)?;
                }
                FileDiffKind::Added | FileDiffKind::Modified => {
                    Self::add_config_file(&path, &etc_upper_dir.0)?;
                    database.0.insert_package_file(
                        uuid.0,
                        &FileEntry {
                            path: path.clone(),
                            is_user: true,
                            scope: FileEntryScope::Config,
                        },
                    )?;
                }
            },
        }

        let remaining = pending.0.len() as u64;
        let processed = total.0 - remaining;
        progress = progress.subject(path).progress(processed, total.0);

        let result = if pending.0.is_empty() {
            StageResult::Advance
        } else {
            StageResult::Repeat
        };

        context.put(pending);
        context.put(tree);
        context.put(database);
        context.put(import_ctx);

        Ok((progress, result, Box::new(NoRollback)))
    }
}

impl ApplyFileStage {
    fn add_file(
        path: &str, repository: &Repository<ObjectID>, tree: &mut FileSystem<ObjectID>, import_ctx: &mut ImportContext,
    ) -> Result<(), FilesError> {
        let source_path = Path::new(path);
        let metadata = symlink_metadata(source_path).map_err(RepoError::from)?;
        let stat = stat_from_metadata(&metadata);
        let handle = FileHandle::new(path);

        handle.ensure_parents_in_tree(tree)?;
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

    fn add_config_file(path: &str, etc_upper_dir: &Path) -> Result<(), FilesError> {
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
