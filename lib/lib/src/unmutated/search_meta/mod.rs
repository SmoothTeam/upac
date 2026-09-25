// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::package::PackageMeta;
use upac_types::request::unmutated::SearchMetaRequest;
use upac_types::response::unmutated::SearchMetaResponse;
use upac_types::state::unmutated::SearchMetaStateId;

use upac_orchestrator::context::Context;
use upac_orchestrator::{Orchestrator, SequentialOrchestrator, run_unmutated, stages};

use self::searching::SearchingStage;

use crate::search::Search;

pub use self::error::SearchMetaError;

mod error;
mod searching;

pub fn run(request: SearchMetaRequest<'_>) -> Result<SearchMetaResponse, (SearchMetaStateId, SearchMetaError)> {
    let cancel_token = unsafe { &*request.base.cancel_token };

    let search = Search::new(request.search, request.is_regex)
        .map_err(|error| (SearchMetaStateId::Setup, SearchMetaError::from(error)))?;

    let mut context = Context::new();
    context.put(search);
    context.put(request.base.message_hook());

    let orchestrator = SequentialOrchestrator::new(stages![SearchingStage]);

    let (metas,) = run_unmutated!(
        orchestrator,
        context,
        cancel_token,
        SearchMetaStateId,
        SearchMetaError,
        Vec<PackageMeta>
    )?;

    Ok(SearchMetaResponse { metas })
}
