// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;

use upac_orchestrator::context::{Context, ctx_get};
use upac_orchestrator::error::PipelineError;
use upac_orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use super::Deploy;
use super::error::PruneError;

pub struct RetentionStage;

impl<E: From<PruneError> + From<PipelineError> + Send + 'static> Stage<E> for RetentionStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), E> {
        let deploy = ctx_get!(context, Deploy);
        deploy.prune_deploys()?;

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
