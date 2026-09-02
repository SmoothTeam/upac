// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::env::temp_dir;
use std::fs::{File, write};

use composefs::generic_tree::Stat;
use composefs::repository::ImportContext;

use upac::composefs::file::FileHandle;
use upac::composefs::repository::commit_tree;
use upac::database::InMemory;
use upac::errors::CommonError;
use upac::layout::database::DATABASE_PATH;
use upac::orchestrator::Context;
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::error::SetupError;
use crate::genesis::{ConfigDigest, ConfigTree, GenesisDatabase, PrefixDigest, PrefixTree};
use crate::layout::genesis::SCRATCH_FILENAME;
use crate::target::TargetSysroot;

pub struct EmbedDatabaseStage;

impl Stage<SetupError> for EmbedDatabaseStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), SetupError> {
        let mut prefix_tree = context.take::<PrefixTree>().ok_or(CommonError::MissingResult)?;
        let config_tree = context.take::<ConfigTree>().ok_or(CommonError::MissingResult)?;
        let database = context.take::<GenesisDatabase>().ok_or(CommonError::MissingResult)?;
        let mut import_ctx = context.take::<ImportContext>().ok_or(CommonError::MissingResult)?;

        let target = context.get::<TargetSysroot>().ok_or(CommonError::MissingResult)?;
        let repository = target.repository();

        let database_bytes = database.0.into_bytes()?;
        let database_scratch_path = temp_dir().join(SCRATCH_FILENAME);
        write(&database_scratch_path, &database_bytes)?;

        FileHandle::new(DATABASE_PATH).insert_file(
            repository,
            &mut prefix_tree.0,
            &File::open(&database_scratch_path)?,
            Stat::uninitialized(),
            &mut import_ctx,
        )?;

        let prefix_digest = commit_tree(repository, prefix_tree.0)?;
        let config_digest = commit_tree(repository, config_tree.0)?;

        context.put(PrefixDigest(prefix_digest));
        context.put(ConfigDigest(config_digest));

        Ok((progress, Box::new(NoRollback::new_none(StageResult::Advance))))
    }
}
