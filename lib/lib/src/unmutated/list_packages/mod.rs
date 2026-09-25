// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::package::PackageMeta;
use upac_types::request::unmutated::ListPackagesRequest;
use upac_types::response::unmutated::ListPackagesResponse;
use upac_types::state::unmutated::ListPackagesStateId;

use upac_orchestrator::context::Context;
use upac_orchestrator::{Orchestrator, SequentialOrchestrator, run_unmutated, stages};

use self::fetching::FetchingStage;

pub use self::error::ListPackagesError;

mod error;
mod fetching;

pub fn run(request: ListPackagesRequest) -> Result<ListPackagesResponse, (ListPackagesStateId, ListPackagesError)> {
    let cancel_token = unsafe { &*request.base.cancel_token };

    let mut context = Context::new();
    context.put(request.base.message_hook());

    let orchestrator = SequentialOrchestrator::new(stages![FetchingStage]);

    let (metas,) = run_unmutated!(
        orchestrator,
        context,
        cancel_token,
        ListPackagesStateId,
        ListPackagesError,
        Vec<PackageMeta>
    )?;

    Ok(ListPackagesResponse { metas })
}
