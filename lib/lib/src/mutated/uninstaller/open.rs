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
use crate::errors::CommonError;
use crate::layout::database::DATABASE_PATH;
use crate::mutated::uninstaller::{
    PackageUuidsToRemove, PendingUuids, TotalPackages, UninstallError, WorkingDatabase, WorkingRemovedConfigPaths,
    WorkingTree,
};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

pub struct OpenTransactionStage;

impl Stage<UninstallError> for OpenTransactionStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), UninstallError> {
        let uuids = context
            .take::<PackageUuidsToRemove>()
            .ok_or(CommonError::MissingResult)?;
        let deploy = context.get::<Deploy>().ok_or(CommonError::MissingResult)?;

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
