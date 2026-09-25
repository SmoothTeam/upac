// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::RequestedPrefixDigestRange;
use upac_types::request::unmutated::DiffPackagesRequest;
use upac_types::response::entry::DiffPackageEntry;
use upac_types::response::unmutated::DiffPackagesResponse;
use upac_types::state::unmutated::DiffPackagesStateId;

use upac_orchestrator::context::Context;
use upac_orchestrator::{Orchestrator, SequentialOrchestrator, run_unmutated, stages};

use self::comparing::ComparingStage;
use self::preparing::PreparingStage;

pub use self::error::DiffPackagesError;

mod comparing;
mod error;
mod preparing;

pub fn run(request: DiffPackagesRequest<'_>) -> Result<DiffPackagesResponse, (DiffPackagesStateId, DiffPackagesError)> {
    let cancel_token = unsafe { &*request.base.cancel_token };

    let mut context = Context::new();
    context.put(RequestedPrefixDigestRange {
        from: request.from_prefix_digest.map(str::to_owned),
        to: request.to_prefix_digest.map(str::to_owned),
    });
    context.put(request.base.message_hook());

    let orchestrator = SequentialOrchestrator::new(stages![PreparingStage, ComparingStage]);

    let (diff_packages,) = run_unmutated!(
        orchestrator,
        context,
        cancel_token,
        DiffPackagesStateId,
        DiffPackagesError,
        Vec<DiffPackageEntry>
    )?;

    Ok(DiffPackagesResponse { diff_packages })
}
