// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::RequestedPrefixDigest;
use upac_types::request::unmutated::ListConfigRequest;
use upac_types::response::entry::ConfigCommitEntry;
use upac_types::response::unmutated::ListConfigResponse;
use upac_types::state::unmutated::ListConfigStateId;

use upac_orchestrator::context::Context;
use upac_orchestrator::{Orchestrator, SequentialOrchestrator, run_unmutated, stages};

use self::fetching::FetchingStage;

pub use self::error::ListConfigError;

mod error;
mod fetching;

pub fn run(request: ListConfigRequest<'_>) -> Result<ListConfigResponse, (ListConfigStateId, ListConfigError)> {
    let cancel_token = unsafe { &*request.base.cancel_token };

    let mut context = Context::new();
    context.put(RequestedPrefixDigest(request.prefix_digest.map(str::to_owned)));
    context.put(request.base.message_hook());

    let orchestrator = SequentialOrchestrator::new(stages![FetchingStage]);

    let (commits,) = run_unmutated!(
        orchestrator,
        context,
        cancel_token,
        ListConfigStateId,
        ListConfigError,
        Vec<ConfigCommitEntry>
    )?;

    Ok(ListConfigResponse { commits })
}
