// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::PathBuf;

use composefs::generic_tree::Stat;
use composefs::repository::ImportContext;
use composefs::tree::FileSystem;

use upac::composefs::file::FileHandle;
use upac::errors::CommonError;
use upac::orchestrator::Context;
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::error::SetupError;
use crate::genesis::{
    ConfigTree, GenesisInput, ImportedConfigPaths, ImportedPrefixPaths, PrefixTree, ResolvedSourceDir,
};
use crate::target::TargetSysroot;

pub struct ImportTreesStage;

impl Stage<SetupError> for ImportTreesStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), SetupError> {
        let target = context.get::<TargetSysroot>().ok_or(CommonError::MissingResult)?;
        let input = context.get::<GenesisInput>().ok_or(CommonError::MissingResult)?;
        let resolved = context.get::<ResolvedSourceDir>().ok_or(CommonError::MissingResult)?;

        let repository = target.repository();
        let mut import_ctx = ImportContext::default();

        let mut prefix_tree = FileSystem::new(Stat::uninitialized());
        let usr_source = resolved.0.join("usr");
        let imported = if usr_source.is_dir() {
            FileHandle::new(PathBuf::new()).import_directory(
                repository,
                &mut prefix_tree,
                &usr_source,
                &mut import_ctx,
                cancel,
            )?
        } else {
            Vec::new()
        };

        let mut config_tree = FileSystem::new(Stat::uninitialized());
        let config_source = resolved.0.join("etc");
        let imported_config = if !input.empty_config && config_source.is_dir() {
            FileHandle::new(PathBuf::new()).import_directory(
                repository,
                &mut config_tree,
                &config_source,
                &mut import_ctx,
                cancel,
            )?
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
