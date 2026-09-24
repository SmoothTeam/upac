// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use composefs::repository::ImportContext;

use upac::orchestrator::context::{Context, ctx_get, ctx_take};
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;

use super::error::BootstrapError;
use super::{PrefixTree, ResolvedSourceDir, import_if_dir};

use crate::layout::genesis::{COMPOSEFS_SETUP_ROOT_UNIT_PATH, SYSTEM_DIR};
use crate::target::TargetSysroot;

pub struct ImportSystemStage;

impl Stage<BootstrapError> for ImportSystemStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), BootstrapError> {
        let mut prefix_tree = ctx_take!(context, PrefixTree);
        let mut imported_ctx = ctx_take!(context, ImportContext);

        let resolved = ctx_get!(context, ResolvedSourceDir);
        let target = ctx_get!(context, TargetSysroot);

        let repository = target.repository();

        let system_dir = resolved.join(SYSTEM_DIR);
        let unit_source = system_dir.join(COMPOSEFS_SETUP_ROOT_UNIT_PATH);
        if !unit_source.is_file() {
            return Err(BootstrapError::ComposefsSetupRootUnitNotFound);
        }

        import_if_dir!(repository, &mut prefix_tree, &system_dir, &mut imported_ctx, cancel);

        context.put(imported_ctx);
        context.put(prefix_tree);

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
