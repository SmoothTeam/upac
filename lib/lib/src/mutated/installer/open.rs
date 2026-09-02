// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::VecDeque;

use composefs::generic_tree::Stat;
use composefs::repository::ImportContext;
use composefs::tree::FileSystem;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use upac_types::{DeclarativeTrigger, PackageTemp};

use crate::composefs::file::FileHandle;
use crate::database::{InMemory, MemoryDatabase};
use crate::deploy::Deploy;
use crate::deploy::digest::current_prefix_digest;
use crate::errors::CommonError;
use crate::layout::database::DATABASE_PATH;
use crate::mutated::installer::{
    ImportedConfigDefaults, ImportedDatabase, ImportedTree, InstallError, PendingPackages, TotalPackages,
};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

pub struct OpenTransactionStage;

impl Stage<InstallError> for OpenTransactionStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), InstallError> {
        let packages = context.take::<Vec<PackageTemp>>().ok_or(CommonError::MissingResult)?;
        let declarative_triggers = context
            .take::<Vec<DeclarativeTrigger>>()
            .ok_or(CommonError::MissingResult)?;
        let deploy = context.get::<Deploy>().ok_or(CommonError::MissingResult)?;

        let current_prefix = current_prefix_digest()?;
        let repository = deploy.open_repository()?;
        let tree = deploy.open_tree(&current_prefix)?;

        let database_bytes = FileHandle::new(DATABASE_PATH).read_file(&repository, &tree)?;
        let database = MemoryDatabase::open_in_memory(database_bytes)?;

        let total = packages.len() as u64;
        let pending: VecDeque<_> = packages.into_iter().zip(declarative_triggers).collect();

        context.put(ImportedTree(tree));
        context.put(ImportedConfigDefaults(FileSystem::new(Stat::uninitialized())));
        context.put(ImportedDatabase(database));
        context.put(ImportContext::default());
        context.put(PendingPackages(pending));
        context.put(TotalPackages(total));

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
