// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::hook::Message;
use upac_types::request::unmutated::SearchInPackageFilesRequest;
use upac_types::response::entry::SearchFileEntry;
use upac_types::response::unmutated::SearchInPackageFilesResponse;
use upac_types::state::unmutated::SearchInPackageFilesStateId;
use upac_types::traits::MessageHook;

use self::searching::SearchingStage;

use crate::orchestrator::context::Context;
use crate::orchestrator::{Orchestrator, SequentialOrchestrator, run_unmutated, stages};
use crate::search::Search;

pub use self::error::SearchInPackageFilesError;

mod error;
mod searching;

pub fn run(
    request: SearchInPackageFilesRequest<'_>,
) -> Result<SearchInPackageFilesResponse, (SearchInPackageFilesStateId, SearchInPackageFilesError)> {
    let cancel_token = unsafe { &*request.base.cancel_token };

    let search = Search::new(request.search, request.is_regex).map_err(|error| {
        (
            SearchInPackageFilesStateId::Setup,
            SearchInPackageFilesError::from(error),
        )
    })?;

    let mut context = Context::new();
    context.put(request.package);
    context.put(search);
    context.put(Box::new(Message::new(request.base.on_hook, request.base.hook_ctx)) as Box<dyn MessageHook>);

    let orchestrator = SequentialOrchestrator::new(stages![SearchingStage]);

    let (files,) = run_unmutated!(
        orchestrator,
        context,
        cancel_token,
        SearchInPackageFilesStateId,
        SearchInPackageFilesError,
        Vec<SearchFileEntry>
    )?;

    Ok(SearchInPackageFilesResponse { files })
}
