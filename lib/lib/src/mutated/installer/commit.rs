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
use crate::layout::database::{DATABASE_PATH, INSTALLER_SCRATCH_FILENAME};
use crate::mutated::installer::{
    ImportedConfigDefaults, ImportedDatabase, ImportedTree, InstallError, NewConfigDefaults, NewPrefixDigest,
};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

pub struct CommitTransactionStage;

impl Stage<InstallError> for CommitTransactionStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), InstallError> {
        let tree = context.take::<ImportedTree>().ok_or(CommonError::MissingResult)?;
        let config_defaults = context
            .take::<ImportedConfigDefaults>()
            .ok_or(CommonError::MissingResult)?;
        let database = context.take::<ImportedDatabase>().ok_or(CommonError::MissingResult)?;
        let mut import_ctx = context.take::<ImportContext>().ok_or(CommonError::MissingResult)?;
        let tmp_path = context.get::<TmpPath>().ok_or(CommonError::MissingResult)?;
        let deploy = context.get::<Deploy>().ok_or(CommonError::MissingResult)?;

        let repository = deploy.open_repository()?;
        let mut tree = tree.0;

        let database_bytes = database.0.into_bytes()?;
        let database_scratch_path = format!("{}/{INSTALLER_SCRATCH_FILENAME}", tmp_path.as_ref());
        write(&database_scratch_path, &database_bytes).map_err(RepoError::from)?;

        FileHandle::new(DATABASE_PATH).insert_file(
            &repository,
            &mut tree,
            &File::open(&database_scratch_path).map_err(RepoError::from)?,
            Stat::uninitialized(),
            &mut import_ctx,
        )?;

        let digest = commit_tree(&repository, tree)?;

        context.put(NewPrefixDigest(digest.to_hex()));
        context.put(NewConfigDefaults(config_defaults.0));

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
