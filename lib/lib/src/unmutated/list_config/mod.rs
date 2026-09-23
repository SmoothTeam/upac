// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::RequestedPrefixDigest;
use upac_types::hook::Message;
use upac_types::request::unmutated::ListConfigRequest;
use upac_types::response::entry::ConfigCommitEntry;
use upac_types::response::unmutated::ListConfigResponse;
use upac_types::state::unmutated::ListConfigStateId;
use upac_types::traits::MessageHook;

use self::fetching::FetchingStage;

use crate::orchestrator::context::Context;
use crate::orchestrator::{Orchestrator, SequentialOrchestrator, run_unmutated, stages};

pub use self::error::ListConfigError;

mod error;
mod fetching;

pub fn run(request: ListConfigRequest<'_>) -> Result<ListConfigResponse, (ListConfigStateId, ListConfigError)> {
    let cancel_token = unsafe { &*request.base.cancel_token };

    let mut context = Context::new();
    context.put(RequestedPrefixDigest(request.prefix_digest.map(str::to_owned)));
    context.put(Box::new(Message::new(request.base.on_hook, request.base.hook_ctx)) as Box<dyn MessageHook>);

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
