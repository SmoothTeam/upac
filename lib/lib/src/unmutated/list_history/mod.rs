// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::hook::Message;
use upac_types::request::unmutated::ListHistoryRequest;
use upac_types::response::entry::HistoryEntry;
use upac_types::response::unmutated::ListHistoryResponse;
use upac_types::state::unmutated::ListHistoryStateId;
use upac_types::traits::MessageHook;

use self::fetching::FetchingStage;

use crate::orchestrator::context::Context;
use crate::orchestrator::{Orchestrator, SequentialOrchestrator, run_unmutated, stages};

pub use self::error::ListHistoryError;

mod error;
mod fetching;

pub fn run(request: ListHistoryRequest) -> Result<ListHistoryResponse, (ListHistoryStateId, ListHistoryError)> {
    let cancel_token = unsafe { &*request.base.cancel_token };

    let mut context = Context::new();
    context.put(Box::new(Message::new(request.base.on_hook, request.base.hook_ctx)) as Box<dyn MessageHook>);

    let orchestrator = SequentialOrchestrator::new(stages![FetchingStage]);

    let (history,) = run_unmutated!(
        orchestrator,
        context,
        cancel_token,
        ListHistoryStateId,
        ListHistoryError,
        Vec<HistoryEntry>
    )?;

    Ok(ListHistoryResponse { history })
}
