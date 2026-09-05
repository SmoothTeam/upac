// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::database::record::DeployRecord;
use crate::deploy::Deploy;
use crate::errors::CommonError;
use crate::mutated::gc::{CollectedRoots, GcError, PendingDeploys, TotalDeploys};
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};
use crate::orchestrator::{Context, ctx_get, ctx_take};

pub struct CollectRootsStage;

impl Stage<GcError> for CollectRootsStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, mut progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), GcError> {
        let mut pending_deploys = ctx_take!(context, PendingDeploys);
        let mut roots = ctx_take!(context, CollectedRoots);

        let total_deploys = ctx_get!(context, TotalDeploys);
        let deploy = ctx_get!(context, Deploy);

        let prefix_digest = pending_deploys.0.pop_front().ok_or(CommonError::MissingResult)?;

        let record = DeployRecord::read(&deploy.deploy(&prefix_digest))?;

        roots.0.push(record.prefix_digest);
        if !record.working_config.is_empty() {
            roots.0.push(record.working_config);
        }
        for entry in record.config_history {
            roots.0.push(entry.config_digest);
        }

        let remaining = pending_deploys.0.len() as u64;
        let processed = total_deploys.0 - remaining;
        progress = progress.subject(prefix_digest).progress(processed, total_deploys.0);

        let stage_result = if pending_deploys.0.is_empty() {
            StageResult::Advance
        } else {
            StageResult::Repeat
        };

        context.put(pending_deploys);
        context.put(roots);

        Ok((progress, stage_result, Box::new(NoRollback)))
    }
}
