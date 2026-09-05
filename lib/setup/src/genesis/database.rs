// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac::database::meta::MetaStoreMut;
use upac::database::{InMemory, MemoryDatabase};
use upac::orchestrator::Context;
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use upac_types::PackageMeta;

use super::ctx_take;

use crate::error::SetupError;
use crate::types::{GenesisDatabase, PackageUuid};

#[cfg(test)]
#[path = "../../tests/inline/database.rs"]
mod tests;

pub struct CreateDatabaseStage;

impl Stage<SetupError> for CreateDatabaseStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), SetupError> {
        let meta = ctx_take!(context, PackageMeta);

        let mut database = MemoryDatabase::new_in_memory()?;
        let uuid = database.insert_package_meta(&meta)?;

        context.put(GenesisDatabase(database));
        context.put(PackageUuid(uuid));

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
