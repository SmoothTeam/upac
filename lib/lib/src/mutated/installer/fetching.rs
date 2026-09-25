// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use super::InstallError;

pub struct FetchingStage;

impl Stage<InstallError> for FetchingStage {
    fn run(
        &self, _context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), InstallError> {
        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
