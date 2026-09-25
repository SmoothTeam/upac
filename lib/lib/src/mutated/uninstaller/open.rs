// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::VecDeque;

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;

use upac_composefs::file::FileHandle;

use upac_database::layout::database::DATABASE_PATH;
use upac_database::{InMemory, MemoryDatabase};

use upac_deploy::Deploy;
use upac_deploy::digest::current_prefix_digest;

use upac_orchestrator::context::{Context, ctx_get, ctx_take};
use upac_orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use super::{PackageUuidsToRemove, RemoveProgress, UninstallError, WorkingState};

pub struct OpenTransactionStage;

impl Stage<UninstallError> for OpenTransactionStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), UninstallError> {
        let uuids = ctx_take!(context, PackageUuidsToRemove);

        let deploy = ctx_get!(context, Deploy);

        let current_prefix = current_prefix_digest()?;
        let repository = deploy.open_repository()?;
        let tree = deploy.open_tree(&current_prefix)?;

        let database_bytes = FileHandle::new(DATABASE_PATH).read_file(&repository, &tree)?;
        let database = MemoryDatabase::open_in_memory(database_bytes)?;

        let total = uuids.len() as u64;
        let pending: VecDeque<_> = uuids.0.into_iter().collect();

        context.put(WorkingState {
            tree,
            database,
            removed_config_paths: Vec::new(),
        });
        context.put(RemoveProgress { total, pending });

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
