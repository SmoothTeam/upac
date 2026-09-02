// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, write};

use composefs::fsverity::FsVerityHashValue;
use composefs::generic_tree::Stat;
use composefs::repository::ImportContext;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use upac_types::TmpPath;

use crate::composefs::error::RepoError;
use crate::composefs::file::FileHandle;
use crate::composefs::repository::commit_tree;
use crate::database::InMemory;
use crate::deploy::Deploy;
use crate::errors::CommonError;
use crate::layout::database::{DATABASE_PATH, UNINSTALL_SCRATCH_FILENAME};
use crate::mutated::uninstaller::{
    NewPrefixDigest, RemovedConfigPaths, UninstallError, WorkingDatabase, WorkingRemovedConfigPaths, WorkingTree,
};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

pub struct CommitTransactionStage;

impl Stage<UninstallError> for CommitTransactionStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), UninstallError> {
        let tree = context.take::<WorkingTree>().ok_or(CommonError::MissingResult)?;
        let database = context.take::<WorkingDatabase>().ok_or(CommonError::MissingResult)?;
        let removed_config_paths = context
            .take::<WorkingRemovedConfigPaths>()
            .ok_or(CommonError::MissingResult)?;
        let tmp_path = context.get::<TmpPath>().ok_or(CommonError::MissingResult)?;
        let deploy = context.get::<Deploy>().ok_or(CommonError::MissingResult)?;

        let repository = deploy.open_repository()?;
        let mut tree = tree.0;

        let database_bytes = database.0.into_bytes()?;
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
        context.put(RemovedConfigPaths(removed_config_paths.0));

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
