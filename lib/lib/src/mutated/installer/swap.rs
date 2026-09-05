// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::mutated::installer::{InstallError, ResolvedBootEntry};
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};
use crate::orchestrator::{Context, ctx_take};

pub struct SwapStage;

impl Stage<InstallError> for SwapStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), InstallError> {
        let resolved = ctx_take!(context, ResolvedBootEntry);

        resolved.plugin.set_one_shot(&resolved.entry_name)?;

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
