// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;

use upac_composefs::file::FileHandle;

use upac_database::layout::database::DATABASE_PATH;
use upac_database::meta::MetaStore;
use upac_database::{InMemory, MemoryDatabase};

use upac_deploy::digest::current_prefix_digest;
use upac_deploy::{Deploy, DeployMode};

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use super::ListPackagesError;

pub struct FetchingStage;

impl Stage<ListPackagesError> for FetchingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), ListPackagesError> {
        let prefix_digest = current_prefix_digest()?;

        let deploy = Deploy::new(DeployMode::ReadOnly)?;
        let repository = deploy.open_repository()?;
        let tree = deploy.open_tree(&prefix_digest)?;

        let database_bytes = FileHandle::new(DATABASE_PATH).read_file(&repository, &tree)?;
        let database = MemoryDatabase::open_in_memory(database_bytes)?;

        context.put(database.list_packages_metas()?);

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
