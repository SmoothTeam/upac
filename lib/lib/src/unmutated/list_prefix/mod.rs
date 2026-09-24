// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::request::unmutated::ListPrefixRequest;
use upac_types::response::entry::PrefixEntry;
use upac_types::response::unmutated::ListPrefixResponse;
use upac_types::state::unmutated::ListPrefixStateId;

use self::fetching::FetchingStage;

use crate::orchestrator::context::Context;
use crate::orchestrator::{Orchestrator, SequentialOrchestrator, run_unmutated, stages};

pub use self::error::ListPrefixError;

mod error;
mod fetching;

pub fn run(request: ListPrefixRequest) -> Result<ListPrefixResponse, (ListPrefixStateId, ListPrefixError)> {
    let cancel_token = unsafe { &*request.base.cancel_token };

    let mut context = Context::new();
    context.put(request.base.message_hook());

    let orchestrator = SequentialOrchestrator::new(stages![FetchingStage]);

    let (prefixes,) = run_unmutated!(
        orchestrator,
        context,
        cancel_token,
        ListPrefixStateId,
        ListPrefixError,
        Vec<PrefixEntry>
    )?;

    Ok(ListPrefixResponse { prefixes })
}
