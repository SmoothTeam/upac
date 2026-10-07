// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::error::ErrorKind;
use upac_types::request::unmutated::SearchMetaRequest;
use upac_types::response::unmutated::SearchMetaResponse;
use upac_types::state::unmutated::SearchMetaStateId;

use upac_deploy::{Sysroot, SysrootMode};

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::searching::SearchingStage;

use crate::Search;

mod searching;

pub fn run(request: SearchMetaRequest<'_>) -> Result<SearchMetaResponse, (SearchMetaStateId, ErrorKind)> {
    let search = Search::new(request.search, request.is_regex).map_err(|error| (SearchMetaStateId::Setup, error))?;
    let sysroot = Sysroot::new(SysrootMode::ReadOnly).map_err(|error| (SearchMetaStateId::Setup, error.into()))?;

    let mut context = Context::default();
    context.put(sysroot);
    context.put(search);

    SequentialOrchestrator::new(stages![SearchingStage]).run_unmutated(
        &mut context,
        request.base.cancel_token,
        &|event| request.base.report_progress(event),
    )
}
