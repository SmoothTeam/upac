// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::composefs::file::FileHandle;
use crate::database::error::DatabaseError;
use crate::database::meta::MetaStore;
use crate::database::{InMemory, MemoryDatabase};
use crate::deploy::digest::current_prefix_digest;
use crate::deploy::{Deploy, DeployMode};
use crate::errors::CommonError;
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage};
use crate::search::Search;
use crate::types::PackageEntry;
use crate::types::database::DATABASE_PATH;
use crate::unmutated::search_in_meta::SearchInMetaError;

pub struct SearchingStage;

impl Stage<SearchInMetaError> for SearchingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), SearchInMetaError> {
        let identity = context.get::<PackageEntry>().ok_or(CommonError::MissingResult)?;
        let search = context.get::<Search>().ok_or(CommonError::MissingResult)?;

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

        Ok((progress, Box::new(NoRollback)))
    }
}
