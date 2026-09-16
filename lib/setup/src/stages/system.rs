// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::Path;

use composefs::generic_tree::Stat;
use composefs::repository::ImportContext;

use upac::composefs::file::FileHandle;
use upac::orchestrator::context::{Context, ctx_get, ctx_take};
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;

use super::{PrefixTree, ResolvedSourceDir, import_if_dir};

use crate::error::SetupError;
use crate::layout::genesis::{
    COMPOSEFS_SETUP_ROOT_UNIT_PATH, COMPOSEFS_SETUP_ROOT_WANTS_PATH, COMPOSEFS_SETUP_ROOT_WANTS_TARGET, SYSTEM_DIR,
};
use crate::target::TargetSysroot;

pub struct ImportSystemStage;

impl Stage<SetupError> for ImportSystemStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), SetupError> {
        let mut prefix_tree = ctx_take!(context, PrefixTree);
        let mut imported_ctx = ctx_take!(context, ImportContext);

        let resolved = ctx_get!(context, ResolvedSourceDir);
        let target = ctx_get!(context, TargetSysroot);

        let repository = target.repository();

        let system_dir = resolved.join(SYSTEM_DIR);
        let unit_source = system_dir.join(COMPOSEFS_SETUP_ROOT_UNIT_PATH);
        if !unit_source.is_file() {
            return Err(SetupError::ComposefsSetupRootUnitNotFound);
        }

        import_if_dir!(repository, &mut prefix_tree, &system_dir, &mut imported_ctx, cancel);

        let mut ancestors: Vec<&Path> = Path::new(COMPOSEFS_SETUP_ROOT_WANTS_PATH)
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

        FileHandle::new(COMPOSEFS_SETUP_ROOT_WANTS_PATH).symlink_in_tree(
            &mut prefix_tree,
            COMPOSEFS_SETUP_ROOT_WANTS_TARGET,
            Stat::uninitialized(),
        )?;

        context.put(imported_ctx);
        context.put(prefix_tree);

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
