// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::read_dir;
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};

use composefs::generic_tree::Stat;
use composefs::repository::ImportContext;
use composefs::tree::FileSystem;

use upac::composefs::file::FileHandle;
use upac::orchestrator::Context;
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use super::ctx_get;

use crate::error::SetupError;
use crate::target::TargetSysroot;
use crate::types::{ConfigTree, GenesisInput, ImportedConfigPaths, ImportedPrefixPaths, PrefixTree, ResolvedSourceDir};

#[cfg(test)]
#[path = "../../tests/inline/trees.rs"]
mod tests;

macro_rules! import_with_progress {
    ($repository:expr, $tree:expr, $source:expr, $import_ctx:expr, $cancel:expr, $context:expr, $stage:expr) => {{
        let total = ImportTreesStage::count_leaf_entries($source).unwrap_or(0);
        let mut current = 0u64;

        FileHandle::new(PathBuf::new()).import_directory(
            $repository,
            $tree,
            $source,
            $import_ctx,
            $cancel,
            &mut |path| {
                current += 1;
                $context.send_progress(
                    &ProgressEventBuilder::new($stage)
                        .subject(path.display().to_string())
                        .progress(current, total),
                );
            },
        )?
    }};
}

pub struct ImportTreesStage;

impl Stage<SetupError> for ImportTreesStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), SetupError> {
        let target = ctx_get!(context, TargetSysroot);
        let input = ctx_get!(context, GenesisInput);
        let resolved = ctx_get!(context, ResolvedSourceDir);

        let repository = target.repository();
        let mut import_ctx = ImportContext::default();
        let stage = progress.stage();

        let mut prefix_tree = FileSystem::new(Stat::uninitialized());
        let prefix_source = resolved.0.join("usr");
        let imported = if prefix_source.is_dir() {
            import_with_progress!(
                repository,
                &mut prefix_tree,
                &prefix_source,
                &mut import_ctx,
                cancel,
                context,
                stage
            )
        } else {
            Vec::new()
        };

        let mut config_tree = FileSystem::new(Stat::uninitialized());
        let config_source = resolved.0.join("etc");
        let imported_config = if !input.empty_config && config_source.is_dir() {
            import_with_progress!(
                repository,
                &mut config_tree,
                &config_source,
                &mut import_ctx,
                cancel,
                context,
                stage
            )
        } else {
            Vec::new()
        };

        context.put(PrefixTree(prefix_tree));
        context.put(ConfigTree(config_tree));
        context.put(ImportedPrefixPaths(imported));
        context.put(ImportedConfigPaths(imported_config));
        context.put(import_ctx);

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}

impl ImportTreesStage {
    fn count_leaf_entries(dir: &Path) -> IoResult<u64> {
        let mut count = 0;

        for entry in read_dir(dir)? {
            let entry = entry?;

            if entry.metadata()?.is_dir() {
                count += Self::count_leaf_entries(&entry.path())?;
            } else {
                count += 1;
            }
        }

        Ok(count)
    }
}
