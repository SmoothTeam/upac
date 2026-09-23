// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::{DiffFileSource, FileDiffKind};

use upac_types::hook::Message;
use upac_types::package::PackageMeta;
use upac_types::request::unmutated::DiffRequest;
use upac_types::response::entry::{DiffPackageEntry, DiffUntrackedFileEntry};
use upac_types::response::unmutated::DiffResponse;
use upac_types::state::unmutated::DiffStateId;
use upac_types::traits::MessageHook;
use upac_types::{RequestedConfigDigestRange, RequestedPrefixDigestRange};

use self::comparing::ComparingStage;
use self::preparing::PreparingStage;

use crate::database::MemoryDatabase;
use crate::orchestrator::context::Context;
use crate::orchestrator::{Orchestrator, SequentialOrchestrator, run_unmutated, stages};

pub use self::error::DiffError;

mod comparing;
mod error;
mod preparing;

struct DiffSnapshot {
    from_packages: Vec<PackageMeta>,
    to_packages: Vec<PackageMeta>,

    changed_files: Vec<(String, FileDiffKind, DiffFileSource)>,

    from_database: MemoryDatabase,
    to_database: MemoryDatabase,
}

pub fn run(request: DiffRequest<'_>) -> Result<DiffResponse, (DiffStateId, DiffError)> {
    let cancel_token = unsafe { &*request.base.cancel_token };

    let mut context = Context::new();
    context.put(RequestedPrefixDigestRange {
        from: request.from_prefix_digest.map(str::to_owned),
        to: request.to_prefix_digest.map(str::to_owned),
    });

    context.put(RequestedConfigDigestRange {
        from: request.from_config_digest.map(str::to_owned),
        to: request.to_config_digest.map(str::to_owned),
    });

    context.put(Box::new(Message::new(request.base.on_hook, request.base.hook_ctx)) as Box<dyn MessageHook>);

    let orchestrator = SequentialOrchestrator::new(stages![PreparingStage, ComparingStage]);

    let (diff_packages, unattached_files) = run_unmutated!(
        orchestrator,
        context,
        cancel_token,
        DiffStateId,
        DiffError,
        Vec<DiffPackageEntry>,
        Vec<DiffUntrackedFileEntry>
    )?;

    Ok(DiffResponse {
        diff_packages,
        unattached_files,
    })
}
