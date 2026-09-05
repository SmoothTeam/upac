// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::VecDeque;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::composefs::file::FileHandle;
use crate::database::{InMemory, MemoryDatabase};
use crate::deploy::Deploy;
use crate::deploy::digest::current_prefix_digest;
use crate::layout::database::DATABASE_PATH;
use crate::mutated::uninstaller::{
    PackageUuidsToRemove, PendingUuids, TotalPackages, UninstallError, WorkingDatabase, WorkingRemovedConfigPaths,
    WorkingTree,
};
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};
use crate::orchestrator::{Context, ctx_get, ctx_take};

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

        let total = uuids.0.len() as u64;
        let pending: VecDeque<_> = uuids.0.into_iter().collect();

        context.put(WorkingTree(tree));
        context.put(WorkingDatabase(database));
        context.put(WorkingRemovedConfigPaths(Vec::new()));
        context.put(PendingUuids(pending));
        context.put(TotalPackages(total));

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
