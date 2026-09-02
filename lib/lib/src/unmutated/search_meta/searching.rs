// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::composefs::file::FileHandle;
use crate::database::meta::MetaStore;
use crate::database::{InMemory, MemoryDatabase};
use crate::deploy::digest::current_prefix_digest;
use crate::deploy::{Deploy, DeployMode};
use crate::errors::CommonError;
use crate::layout::database::DATABASE_PATH;
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};
use crate::search::Search;
use crate::unmutated::search_meta::SearchMetaError;

pub struct SearchingStage;

impl Stage<SearchMetaError> for SearchingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), SearchMetaError> {
        let search = context.get::<Search>().ok_or(CommonError::MissingResult)?;

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
