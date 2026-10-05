// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::error::ErrorKind;
use upac_types::request::unmutated::ListConfigRequest;
use upac_types::response::unmutated::ListConfigResponse;
use upac_types::state::unmutated::ListConfigStateId;

use upac_composefs::Digest;

use upac_deploy::{Sysroot, SysrootMode};

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::fetching::FetchingStage;

use crate::report_progress;

mod fetching;

pub(crate) struct RequestedPrefixDigest(pub Option<Digest>);

pub fn run(request: ListConfigRequest<'_>) -> Result<ListConfigResponse, (ListConfigStateId, ErrorKind)> {
    let requested_prefix_digest = request
        .prefix_digest
        .map(Digest::from_hex)
        .transpose()
        .map_err(|error| (ListConfigStateId::Setup, error.into()))?;
    let sysroot = Sysroot::new(SysrootMode::ReadOnly).map_err(|error| (ListConfigStateId::Setup, error.into()))?;

    let mut context = Context::default();
    context.put(sysroot);
    context.put(RequestedPrefixDigest(requested_prefix_digest));

    SequentialOrchestrator::new(stages![FetchingStage]).run_unmutated(
        &mut context,
        request.base.cancel_token,
        &|event| report_progress(&request.base, event),
    )
}
