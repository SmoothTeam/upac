// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::CancelToken;
use upac_types::hook::ProgressEventBuilder;

use upac_composefs::repository::gc;

use upac_deploy::Deploy;

use upac_orchestrator::context::{Context, ctx_take};
use upac_orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use super::{CollectedRoots, GcError};

pub struct CleaningStage;

impl Stage<GcError> for CleaningStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), GcError> {
        let roots = ctx_take!(context, CollectedRoots);
        let deploy = ctx_take!(context, Deploy);

        let repository = deploy.open_repository()?;
        let root_refs: Vec<&str> = roots.iter().map(String::as_str).collect();
        gc(&repository, &root_refs)?;

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
