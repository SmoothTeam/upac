// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::CancelToken;

use upac_types::UninstallPackagesTargets;
use upac_types::decoder::DeclarativeTrigger;
use upac_types::hook::ProgressEventBuilder;

use upac_composefs::file::FileHandle;

use upac_database::layout::database::DATABASE_PATH;
use upac_database::meta::MetaStore;
use upac_database::triggers::TriggerStore;
use upac_database::{InMemory, MemoryDatabase};

use upac_deploy::Deploy;
use upac_deploy::digest::current_prefix_digest;

use upac_orchestrator::context::{Context, ctx_get};
use upac_orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use super::{PackageUuidsToRemove, UninstallError};

pub struct PreparationStage;

impl Stage<UninstallError> for PreparationStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), UninstallError> {
        let targets = ctx_get!(context, UninstallPackagesTargets);
        let deploy = ctx_get!(context, Deploy);

        let current_prefix = current_prefix_digest()?;
        let repository = deploy.open_repository()?;
        let tree = deploy.open_tree(&current_prefix)?;

        let database_bytes = FileHandle::new(DATABASE_PATH).read_file(&repository, &tree)?;
        let database = MemoryDatabase::open_in_memory(database_bytes)?;

        let mut uuids = Vec::new();
        let mut declarative_triggers: Vec<DeclarativeTrigger> = Vec::new();
        for entry in &targets.0 {
            let uuid = database
                .find_package_uuid(&entry.name, &entry.arch, entry.arch_sub.as_deref())?
                .ok_or(UninstallError::PackageNotFound)?;
            uuids.push(uuid);

            if let Some(trigger) = database.get_declarative_triggers(uuid)? {
                declarative_triggers.push(trigger);
            }
        }

        context.put(PackageUuidsToRemove(uuids));
        context.put(declarative_triggers);

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
