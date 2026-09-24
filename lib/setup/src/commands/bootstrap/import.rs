// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::Path;

use composefs::repository::ImportContext;

use upac::database::files::FileStoreMut;
use upac::database::meta::MetaStoreMut;
use upac::database::triggers::TriggerStoreMut;
use upac::errors::CommonError;
use upac::orchestrator::context::{Context, ctx_get, ctx_take};
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;
use upac_types::response::entry::{FileEntry, FileEntryScope};

use super::error::BootstrapError;
use super::{ConfigState, EmptyConfig, PrefixTree, SetupProgress, import_if_dir};

use crate::target::TargetSysroot;

pub struct ImportPackageStage;

impl Stage<BootstrapError> for ImportPackageStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, mut progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), BootstrapError> {
        let mut setup_progress = ctx_take!(context, SetupProgress);
        let mut imported_ctx = ctx_take!(context, ImportContext);
        let mut prefix_tree = ctx_take!(context, PrefixTree);
        let mut config_state = ctx_take!(context, ConfigState);

        let target = ctx_get!(context, TargetSysroot);
        let empty_config = ctx_get!(context, EmptyConfig);

        let repository = target.repository();

        let (package, trigger) = setup_progress.pending.pop_front().ok_or(CommonError::MissingResult)?;

        let source_root = Path::new(&package.temp_package_path);

        let prefix_source = source_root.join("usr");
        let imported = import_if_dir!(repository, &mut prefix_tree, &prefix_source, &mut imported_ctx, cancel);

        let config_source = source_root.join("etc");
        let imported_config = if **empty_config {
            Vec::new()
        } else {
            import_if_dir!(
                repository,
                &mut config_state.config_tree,
                &config_source,
                &mut imported_ctx,
                cancel
            )
        };

        let uuid = config_state.database.insert_package_meta(&package.meta)?;
        config_state.database.set_declarative_triggers(uuid, &trigger)?;

        for path in imported {
            config_state.database.insert_package_file(
                uuid,
                &FileEntry {
                    path: path.to_string_lossy().into_owned(),
                    is_user: false,
                    scope: FileEntryScope::Prefix,
                },
            )?;
        }

        for path in imported_config {
            config_state.database.insert_package_file(
                uuid,
                &FileEntry {
                    path: path.to_string_lossy().into_owned(),
                    is_user: false,
                    scope: FileEntryScope::Config,
                },
            )?;
        }

        let remaining = setup_progress.pending.len() as u64;
        let processed = setup_progress.total - remaining;
        progress = progress
            .subject(package.meta.name.clone())
            .progress(processed, setup_progress.total);

        let result = if setup_progress.pending.is_empty() {
            StageResult::Advance
        } else {
            StageResult::Repeat
        };

        context.put(setup_progress);
        context.put(imported_ctx);
        context.put(config_state);
        context.put(prefix_tree);

        Ok((progress, result, Box::new(NoRollback)))
    }
}
