// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::database::record::DeployRecord;
use crate::deploy::Deploy;
use crate::errors::CommonError;
use crate::mutated::gc::{CollectedRoots, GcError, PendingDeploys, TotalDeploys};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

pub struct CollectRootsStage;

impl Stage<GcError> for CollectRootsStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, mut progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), GcError> {
        let mut pending = context.take::<PendingDeploys>().ok_or(CommonError::MissingResult)?;
        let mut roots = context.take::<CollectedRoots>().ok_or(CommonError::MissingResult)?;
        let total = context.get::<TotalDeploys>().ok_or(CommonError::MissingResult)?;
        let deploy = context.get::<Deploy>().ok_or(CommonError::MissingResult)?;

        let prefix_digest = pending.0.pop_front().ok_or(CommonError::MissingResult)?;

        let record = DeployRecord::read(&deploy.deploy(&prefix_digest))?;

        roots.0.push(record.prefix_digest);
        if !record.working_config.is_empty() {
            roots.0.push(record.working_config);
        }
        for entry in record.config_history {
            roots.0.push(entry.config_digest);
        }

        let remaining = pending.0.len() as u64;
        let processed = total.0 - remaining;
        progress = progress.subject(prefix_digest).progress(processed, total.0);

        let result = if pending.0.is_empty() {
            StageResult::Advance
        } else {
            StageResult::Repeat
        };

        context.put(pending);
        context.put(roots);

        Ok((progress, result, Box::new(NoRollback)))
    }
}
