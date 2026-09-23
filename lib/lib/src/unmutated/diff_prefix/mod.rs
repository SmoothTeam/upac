// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::FileDiffKind;

use upac_types::RequestedPrefixDigestRange;
use upac_types::hook::Message;
use upac_types::request::unmutated::DiffPrefixRequest;
use upac_types::response::entry::DiffPrefixFileEntry;
use upac_types::response::unmutated::DiffPrefixResponse;
use upac_types::state::unmutated::DiffPrefixStateId;
use upac_types::traits::MessageHook;

use self::comparing::ComparingStage;
use self::preparing::PreparingStage;

use crate::database::MemoryDatabase;
use crate::orchestrator::context::Context;
use crate::orchestrator::{Orchestrator, SequentialOrchestrator, run_unmutated, stages};

pub use self::error::DiffPrefixError;

mod comparing;
mod error;
mod preparing;

struct DiffPrefixSnapshot {
    changed: Vec<(String, FileDiffKind)>,
    from_database: MemoryDatabase,
    to_database: MemoryDatabase,
}

pub fn run(request: DiffPrefixRequest<'_>) -> Result<DiffPrefixResponse, (DiffPrefixStateId, DiffPrefixError)> {
    let cancel_token = unsafe { &*request.base.cancel_token };

    let mut context = Context::new();
    context.put(RequestedPrefixDigestRange {
        from: request.from_prefix_digest.map(str::to_owned),
        to: request.to_prefix_digest.map(str::to_owned),
    });
    context.put(Box::new(Message::new(request.base.on_hook, request.base.hook_ctx)) as Box<dyn MessageHook>);

    let orchestrator = SequentialOrchestrator::new(stages![PreparingStage, ComparingStage]);

    let (files,) = run_unmutated!(
        orchestrator,
        context,
        cancel_token,
        DiffPrefixStateId,
        DiffPrefixError,
        Vec<DiffPrefixFileEntry>
    )?;

    Ok(DiffPrefixResponse { files })
}
