// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::package::PackageMeta;
use upac_types::request::unmutated::SearchInMetaRequest;
use upac_types::response::unmutated::SearchInMetaResponse;
use upac_types::state::unmutated::SearchInMetaStateId;

use self::searching::SearchingStage;

use crate::orchestrator::context::Context;
use crate::orchestrator::{Orchestrator, SequentialOrchestrator, run_unmutated, stages};
use crate::search::Search;

pub use self::error::SearchInMetaError;

mod error;
mod searching;

pub fn run(request: SearchInMetaRequest<'_>) -> Result<SearchInMetaResponse, (SearchInMetaStateId, SearchInMetaError)> {
    let cancel_token = unsafe { &*request.base.cancel_token };

    let search = Search::new(request.search, request.is_regex)
        .map_err(|error| (SearchInMetaStateId::Setup, SearchInMetaError::from(error)))?;

    let mut context = Context::new();
    context.put(request.package);
    context.put(search);
    context.put(request.base.message_hook());

    let orchestrator = SequentialOrchestrator::new(stages![SearchingStage]);

    let (metas,) = run_unmutated!(
        orchestrator,
        context,
        cancel_token,
        SearchInMetaStateId,
        SearchInMetaError,
        Vec<PackageMeta>
    )?;

    Ok(SearchInMetaResponse { metas })
}
