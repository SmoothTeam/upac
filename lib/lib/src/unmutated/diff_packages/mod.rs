// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::error::ErrorKind;
use upac_types::package::PackageMeta;
use upac_types::request::unmutated::DiffPackagesRequest;
use upac_types::response::unmutated::DiffPackagesResponse;
use upac_types::state::unmutated::DiffPackagesStateId;

use upac_deploy::{Sysroot, SysrootMode};

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::comparing::ComparingStage;
use self::preparing::PreparingStage;

use super::RequestedPrefixDigestRange;

mod comparing;
mod preparing;

struct DiffPackagesSnapshot {
    from: Vec<PackageMeta>,
    to: Vec<PackageMeta>,
}

pub fn run(request: DiffPackagesRequest<'_>) -> Result<DiffPackagesResponse, (DiffPackagesStateId, ErrorKind)> {
    let requested = RequestedPrefixDigestRange::parse(request.from_prefix_digest, request.to_prefix_digest)
        .map_err(|error| (DiffPackagesStateId::Setup, error))?;
    let sysroot = Sysroot::new(SysrootMode::ReadOnly).map_err(|error| (DiffPackagesStateId::Setup, error.into()))?;

    let mut context = Context::default();
    context.put(sysroot);
    context.put(requested);

    SequentialOrchestrator::new(stages![PreparingStage, ComparingStage]).run_unmutated(
        &mut context,
        request.base.cancel_token,
        &|event| request.base.report_progress(event),
    )
}
