// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac::orchestrator::context::Context;
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;

use crate::error::SetupError;

pub struct KernelStage;

impl Stage<SetupError> for KernelStage {
    fn run(
        &self, _context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), SetupError> {
        todo!();

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
