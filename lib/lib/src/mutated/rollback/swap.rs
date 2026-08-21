// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::errors::CommonError;
use crate::mutated::rollback::{ResolvedBootEntry, RollbackError};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage};

pub struct SwapStage;

impl Stage<RollbackError> for SwapStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), RollbackError> {
        let resolved = context.take::<ResolvedBootEntry>().ok_or(CommonError::MissingResult)?;

        resolved.plugin.set_one_shot(&resolved.entry_name)?;

        Ok((progress, Box::new(NoRollback)))
    }
}
