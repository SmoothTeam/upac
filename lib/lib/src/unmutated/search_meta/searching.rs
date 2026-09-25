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

use upac_orchestrator::context::{Context, ctx_get};
use upac_orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use super::SearchMetaError;

use crate::search::Search;

pub struct SearchingStage;

impl Stage<SearchMetaError> for SearchingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), SearchMetaError> {
        let search = ctx_get!(context, Search);

        let prefix_digest = current_prefix_digest()?;

        let deploy = Deploy::new(DeployMode::ReadOnly)?;
        let repository = deploy.open_repository()?;

        let tree = deploy.open_tree(&prefix_digest)?;

        let database_bytes = FileHandle::new(DATABASE_PATH).read_file(&repository, &tree)?;
        let database = MemoryDatabase::open_in_memory(database_bytes)?;

        let matches: Vec<_> = database
            .list_packages_metas()?
            .into_iter()
            .filter(|meta| search.is_match(&meta.name) || search.is_match(&meta.description))
            .collect();

        context.put(matches);

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
