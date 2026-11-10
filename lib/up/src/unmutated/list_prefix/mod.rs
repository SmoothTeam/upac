// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::error::ErrorKind;
use upac_types::request::unmutated::ListPrefixRequest;
use upac_types::response::unmutated::ListPrefixResponse;
use upac_types::state::unmutated::ListPrefixStateId;

use upac_deploy::{Sysroot, SysrootMode};

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::fetching::FetchingStage;

mod fetching;

pub fn run(
    request: ListPrefixRequest<'_>,
) -> Result<ListPrefixResponse, (ListPrefixStateId, ErrorKind, Option<String>)> {
    let sysroot =
        Sysroot::new(SysrootMode::ReadOnly).map_err(|error| (ListPrefixStateId::Setup, error.into(), None))?;

    let mut context = Context::default();
    context.put(sysroot);

    SequentialOrchestrator::new(stages![FetchingStage]).run_unmutated(
        &mut context,
        request.base.cancel_token,
        &|event| request.base.report_progress(event),
    )
}
