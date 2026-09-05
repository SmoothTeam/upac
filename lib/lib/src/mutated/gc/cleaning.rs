// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::composefs::repository::gc;
use crate::deploy::Deploy;
use crate::mutated::gc::{CollectedRoots, GcError};
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};
use crate::orchestrator::{Context, ctx_take};

pub struct CleaningStage;

impl Stage<GcError> for CleaningStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), GcError> {
        let roots = ctx_take!(context, CollectedRoots);
        let deploy = ctx_take!(context, Deploy);

        let repository = deploy.open_repository()?;
        let root_refs: Vec<&str> = roots.0.iter().map(String::as_str).collect();
        gc(&repository, &root_refs)?;

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
