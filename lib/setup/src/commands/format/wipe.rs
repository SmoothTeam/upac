// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac::orchestrator::context::{Context, ctx_get};
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;

use super::error::FormatError;
use super::{ForceWipe, RequestedDevice};

use crate::wipe::WipeTarget;

pub struct WipeStage;

impl Stage<FormatError> for WipeStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), FormatError> {
        let device_path = ctx_get!(context, RequestedDevice);
        let force_wipe = ctx_get!(context, ForceWipe);

        WipeTarget { device_path }.wipe_or_refuse(**force_wipe)?;

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
