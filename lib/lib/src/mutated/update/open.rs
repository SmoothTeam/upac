// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use composefs::generic_tree::Stat;
use composefs::repository::ImportContext;
use composefs::tree::FileSystem;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::composefs::file::FileHandle;
use crate::database::{InMemory, MemoryDatabase};
use crate::deploy::Deploy;
use crate::deploy::digest::current_prefix_digest;
use crate::layout::database::DATABASE_PATH;
use crate::mutated::update::{
    ImportedConfigDefaults, ImportedDatabase, ImportedRemovedConfigPaths, ImportedTree, UpdateError,
};
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};
use crate::orchestrator::{Context, ctx_get};

pub struct OpenTransactionStage;

impl Stage<UpdateError> for OpenTransactionStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), UpdateError> {
        let deploy = ctx_get!(context, Deploy);

        let current_prefix = current_prefix_digest()?;
        let repository = deploy.open_repository()?;
        let tree = deploy.open_tree(&current_prefix)?;

        let database_bytes = FileHandle::new(DATABASE_PATH).read_file(&repository, &tree)?;
        let database = MemoryDatabase::open_in_memory(database_bytes)?;

        context.put(ImportedTree(tree));
        context.put(ImportedConfigDefaults(FileSystem::new(Stat::uninitialized())));
        context.put(ImportedDatabase(database));
        context.put(ImportedRemovedConfigPaths(Vec::new()));
        context.put(ImportContext::default());

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
