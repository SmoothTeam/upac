// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::composefs::file::FileHandle;
use crate::database::files::FileStore;
use crate::database::meta::MetaStore;
use crate::database::{InMemory, MemoryDatabase};
use crate::deploy::digest::current_prefix_digest;
use crate::deploy::{Deploy, DeployMode};
use crate::errors::CommonError;
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage};
use crate::search::Search;
use crate::types::SearchFileEntry;
use crate::types::database::DATABASE_PATH;
use crate::unmutated::search_files::SearchFilesError;

pub struct SearchingStage;

impl Stage<SearchFilesError> for SearchingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), SearchFilesError> {
        let search = context.get::<Search>().ok_or(CommonError::MissingResult)?;

        let prefix_digest = current_prefix_digest()?;

        let deploy = Deploy::new(DeployMode::ReadOnly)?;
        let repository = deploy.open_repository()?;

        let tree = deploy.open_tree(&prefix_digest)?;

        let database_bytes = FileHandle::new(DATABASE_PATH).read_file(&repository, &tree)?;
        let database = MemoryDatabase::open_in_memory(database_bytes)?;

        let mut matches = Vec::new();

        for (uuid, file_entry) in database.list_files()? {
            if !search.is_match(&file_entry.path) {
                continue;
            }

            let Some(package_meta) = database.get_package_meta(uuid)? else {
                continue;
            };

            matches.push(SearchFileEntry {
                path: file_entry.path,
                package_name: package_meta.name,
                is_user: file_entry.is_user,
            });
        }

        context.put(matches);

        Ok((progress, Box::new(NoRollback)))
    }
}
