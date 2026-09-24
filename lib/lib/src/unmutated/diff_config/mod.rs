// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::FileDiffKind;

use upac_types::RequestedConfigDigestRange;
use upac_types::request::unmutated::DiffConfigRequest;
use upac_types::response::entry::DiffConfigFileEntry;
use upac_types::response::unmutated::DiffConfigResponse;
use upac_types::state::unmutated::DiffConfigStateId;

use self::comparing::ComparingStage;
use self::preparing::PreparingStage;

use crate::database::MemoryDatabase;
use crate::orchestrator::context::Context;
use crate::orchestrator::{Orchestrator, SequentialOrchestrator, run_unmutated, stages};

pub use self::error::DiffConfigError;

mod comparing;
mod error;
mod preparing;

struct DiffConfigSnapshot {
    changed: Vec<(String, FileDiffKind)>,
    from_database: MemoryDatabase,
    to_database: MemoryDatabase,
}

pub fn run(request: DiffConfigRequest<'_>) -> Result<DiffConfigResponse, (DiffConfigStateId, DiffConfigError)> {
    let cancel_token = unsafe { &*request.base.cancel_token };

    let mut context = Context::new();
    context.put(RequestedConfigDigestRange {
        from: request.from_config_digest.map(str::to_owned),
        to: request.to_config_digest.map(str::to_owned),
    });
    context.put(request.base.message_hook());

    let orchestrator = SequentialOrchestrator::new(stages![PreparingStage, ComparingStage]);

    let (files,) = run_unmutated!(
        orchestrator,
        context,
        cancel_token,
        DiffConfigStateId,
        DiffConfigError,
        Vec<DiffConfigFileEntry>
    )?;

    Ok(DiffConfigResponse { files })
}
