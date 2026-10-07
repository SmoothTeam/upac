// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::error::ErrorKind;
use upac_types::request::unmutated::SearchInMetaRequest;
use upac_types::response::unmutated::SearchInMetaResponse;
use upac_types::state::unmutated::SearchInMetaStateId;

use upac_deploy::{Sysroot, SysrootMode};

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::searching::SearchingStage;

use crate::Search;

mod searching;

pub fn run(request: SearchInMetaRequest<'_>) -> Result<SearchInMetaResponse, (SearchInMetaStateId, ErrorKind)> {
    let search = Search::new(request.search, request.is_regex).map_err(|error| (SearchInMetaStateId::Setup, error))?;
    let sysroot = Sysroot::new(SysrootMode::ReadOnly).map_err(|error| (SearchInMetaStateId::Setup, error.into()))?;

    let mut context = Context::default();
    context.put(sysroot);
    context.put(search);
    context.put(request.package);

    SequentialOrchestrator::new(stages![SearchingStage]).run_unmutated(
        &mut context,
        request.base.cancel_token,
        &|event| request.base.report_progress(event),
    )
}
