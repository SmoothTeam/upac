// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::VecDeque;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::deploy::Deploy;
use crate::errors::CommonError;
use crate::mutated::gc::{CollectedRoots, GcError, PendingDeploys, TotalDeploys};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

pub struct PruneStage;

impl Stage<GcError> for PruneStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), GcError> {
        let deploy = context.get::<Deploy>().ok_or(CommonError::MissingResult)?;

        deploy.prune_deploys()?;

        let deploys = deploy.deploys()?;
        let total = deploys.len() as u64;
        let pending: VecDeque<_> = deploys.into_iter().collect();

        context.put(PendingDeploys(pending));
        context.put(TotalDeploys(total));
        context.put(CollectedRoots(Vec::new()));

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
