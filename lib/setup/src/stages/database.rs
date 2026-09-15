// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::env::temp_dir;
use std::fs::{File, write};
use std::path::Path;

use composefs::generic_tree::Stat;
use composefs::repository::ImportContext;

use upac::composefs::file::FileHandle;
use upac::composefs::repository::commit_tree;
use upac::database::InMemory;
use upac::layout::database::DATABASE_PATH;
use upac::orchestrator::context::{Context, ctx_get, ctx_take};
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;

use super::{ConfigState, DeployDigests, PrefixTree};

use crate::error::SetupError;
use crate::layout::genesis::SCRATCH_FILENAME;
use crate::target::TargetSysroot;

#[cfg(test)]
#[path = "../../tests/inline/embed.rs"]
mod tests;

pub struct EmbedDatabaseStage;

impl Stage<SetupError> for EmbedDatabaseStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), SetupError> {
        let mut import_ctx = ctx_take!(context, ImportContext);
        let mut config_state = ctx_take!(context, ConfigState);
        let mut prefix_tree = ctx_take!(context, PrefixTree);

        let target = ctx_get!(context, TargetSysroot);

        let repository = target.repository();

        let database_bytes = config_state.database.into_bytes()?;
        let database_scratch_path = temp_dir().join(SCRATCH_FILENAME);
        write(&database_scratch_path, &database_bytes)?;

        let mut ancestors: Vec<&Path> = Path::new(DATABASE_PATH)
            .ancestors()
            .skip(1)
            .filter(|ancestor| !ancestor.as_os_str().is_empty())
            .collect();
        ancestors.reverse();

        for ancestor in ancestors {
            let handle = FileHandle::new(ancestor);
            if handle.stat_in_tree(&prefix_tree).is_err() {
                handle.insert_in_tree(&mut prefix_tree, Stat::uninitialized())?;
            }
        }

        let database_handle = FileHandle::new(DATABASE_PATH);
        database_handle.insert_file(
            repository,
            &mut prefix_tree,
            &File::open(&database_scratch_path)?,
            Stat::uninitialized(),
            &mut import_ctx,
        )?;

        let prefix_digest = commit_tree(repository, *prefix_tree)?;
        let config_digest = commit_tree(repository, config_state.config_tree)?;

        context.put(DeployDigests {
            prefix: prefix_digest,
            config: config_digest,
        });

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
