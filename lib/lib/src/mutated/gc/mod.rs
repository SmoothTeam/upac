// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::error::ErrorKind;
use upac_types::request::mutated::GcRequest;
use upac_types::settings::RuntimeSettings;
use upac_types::state::mutated::GcStateId;

use upac_composefs::Digest;

use upac_deploy::deployment::PrefixDeploy;
use upac_deploy::{Sysroot, SysrootMode};

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::cleaning::CleaningStage;
use self::collect::CollectRootsStage;
use self::pruning::PruneStage;

mod cleaning;
mod collect;
mod pruning;

pub(crate) struct RetainedPrefixes(pub Vec<PrefixDeploy>);

pub(crate) struct GcRoots(pub Vec<Digest>);

pub fn run(request: GcRequest<'_>) -> Result<(), (GcStateId, ErrorKind)> {
    let sysroot = Sysroot::new(SysrootMode::ReadWrite).map_err(|error| (GcStateId::Setup, error.into()))?;

    let mut context = Context::default();
    context.put(sysroot);

    SequentialOrchestrator::new(stages![
        PruneStage {
            retention_depth: RuntimeSettings::load().gc.retention_depth,
        },
        CollectRootsStage,
        CleaningStage,
    ])
    .run_mutating(&mut context, request.base.cancel_token, &|event| {
        request.base.report_progress(event)
    })
}
