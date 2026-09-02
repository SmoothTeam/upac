// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac::database::meta::MetaStoreMut;
use upac::database::{InMemory, MemoryDatabase};
use upac::errors::CommonError;
use upac::orchestrator::Context;
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use upac_types::PackageMeta;

use crate::error::SetupError;
use crate::genesis::{GenesisDatabase, PackageUuid};

pub struct CreateDatabaseStage;

impl Stage<SetupError> for CreateDatabaseStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), SetupError> {
        let meta = context.take::<PackageMeta>().ok_or(CommonError::MissingResult)?;

        let mut database = MemoryDatabase::new_in_memory()?;
        let uuid = database.insert_package_meta(&meta)?;

        context.put(GenesisDatabase(database));
        context.put(PackageUuid(uuid));

        Ok((progress, Box::new(NoRollback::new_none(StageResult::Advance))))
    }
}
