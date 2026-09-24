// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac::errors::CommonError;
use upac::orchestrator::context::{Context, ctx_get, ctx_take};
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;

use upac_types::TmpPath;

use super::error::BootstrapError;
use super::{SetupProgress, UnpackState};

pub struct UnpackPackageStage;

impl Stage<BootstrapError> for UnpackPackageStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, mut progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), BootstrapError> {
        let mut setup_progress = ctx_take!(context, SetupProgress);
        let mut unpack_state = ctx_take!(context, UnpackState);

        let tmp_path = ctx_get!(context, TmpPath);

        let package_path = unpack_state
            .pending_paths
            .pop_front()
            .ok_or(CommonError::MissingResult)?;
        let index = setup_progress.pending.len();

        let (package, trigger) = unpack_state
            .unpacker
            .unpack_one(&package_path, index, tmp_path.as_ref(), cancel)
            .map_err(CommonError::Decoder)?;

        setup_progress.pending.push_back((package, trigger));

        let remaining = setup_progress.pending.len() as u64;
        let processed = setup_progress.total - remaining;
        progress = progress.subject(package_path).progress(processed, setup_progress.total);

        let result = if unpack_state.pending_paths.is_empty() {
            StageResult::Advance
        } else {
            StageResult::Repeat
        };

        context.put(setup_progress);
        context.put(unpack_state);

        Ok((progress, result, Box::new(NoRollback)))
    }
}
