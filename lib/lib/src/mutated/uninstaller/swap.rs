// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::errors::CommonError;
use crate::mutated::uninstaller::{ResolvedBootEntry, UninstallError};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

pub struct SwapStage;

impl Stage<UninstallError> for SwapStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), UninstallError> {
        let resolved = context.take::<ResolvedBootEntry>().ok_or(CommonError::MissingResult)?;

        resolved.plugin.set_one_shot(&resolved.entry_name)?;

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
