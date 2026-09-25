// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;
use upac_types::package::PackageInfo;

use upac_composefs::file::FileHandle;

use upac_database::error::DatabaseError;
use upac_database::layout::database::DATABASE_PATH;
use upac_database::meta::MetaStore;
use upac_database::{InMemory, MemoryDatabase};

use upac_deploy::digest::current_prefix_digest;
use upac_deploy::{Deploy, DeployMode};

use upac_orchestrator::context::{Context, ctx_get};
use upac_orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use super::SearchInMetaError;

use crate::search::Search;

pub struct SearchingStage;

impl Stage<SearchInMetaError> for SearchingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), SearchInMetaError> {
        let identity = ctx_get!(context, PackageInfo);
        let search = ctx_get!(context, Search);

        let prefix_digest = current_prefix_digest()?;

        let deploy = Deploy::new(DeployMode::ReadOnly)?;
        let repository = deploy.open_repository()?;

        let tree = deploy.open_tree(&prefix_digest)?;

        let database_bytes = FileHandle::new(DATABASE_PATH).read_file(&repository, &tree)?;
        let database = MemoryDatabase::open_in_memory(database_bytes)?;

        let uuid = database
            .find_package_uuid(&identity.name, &identity.arch, identity.arch_sub.as_deref())?
            .ok_or(DatabaseError::PackageNotFound)?;
        let package_meta = database.get_package_meta(uuid)?.ok_or(DatabaseError::PackageNotFound)?;

        let matches = if search.is_match(&package_meta.name) || search.is_match(&package_meta.description) {
            vec![package_meta]
        } else {
            Vec::new()
        };

        context.put(matches);

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
