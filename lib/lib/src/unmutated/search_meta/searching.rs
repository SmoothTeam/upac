// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::composefs::file::FileHandle;
use crate::database::meta::MetaStore;
use crate::database::{InMemory, MemoryDatabase};
use crate::deploy::digest::current_prefix_digest;
use crate::deploy::{Deploy, DeployMode};
use crate::errors::CommonError;
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage};
use crate::types::Search;
use crate::types::database::DATABASE_PATH;
use crate::unmutated::search_meta::SearchMetaError;

pub struct SearchingStage;

impl Stage<SearchMetaError> for SearchingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), SearchMetaError> {
        let search = context.get::<Search>().ok_or(CommonError::MissingResult)?;
        let needle = search.as_ref().to_lowercase();

        let prefix_digest = current_prefix_digest()?;

        let deploy = Deploy::new(DeployMode::ReadOnly)?;
        let repository = deploy.open_repository()?;

        let tree = deploy.open_tree(&prefix_digest)?;

        let database_bytes = FileHandle::new(DATABASE_PATH).read_file(&repository, &tree)?;
        let database = MemoryDatabase::open_in_memory(database_bytes)?;

        let matches: Vec<_> = database
            .list_packages_metas()?
            .into_iter()
            .filter(|meta| {
                meta.name.to_lowercase().contains(&needle) || meta.description.to_lowercase().contains(&needle)
            })
            .collect();

        context.put(matches);

        Ok((progress, Box::new(NoRollback)))
    }
}
