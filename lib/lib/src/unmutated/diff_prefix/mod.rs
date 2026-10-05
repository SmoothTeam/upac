// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::BTreeMap;
use std::path::PathBuf;

use upac_types::diff::FileDiffKind;
use upac_types::error::ErrorKind;
use upac_types::request::unmutated::DiffPrefixRequest;
use upac_types::response::unmutated::DiffPrefixResponse;
use upac_types::state::unmutated::DiffPrefixStateId;

use upac_database::MemoryDatabase;

use upac_deploy::{Sysroot, SysrootMode};

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::comparing::ComparingStage;
use self::preparing::PreparingStage;

use super::RequestedPrefixDigestRange;

use crate::report_progress;

mod comparing;
mod preparing;

struct DiffPrefixSnapshot {
    changed: BTreeMap<PathBuf, FileDiffKind>,
    from_database: MemoryDatabase,
    to_database: MemoryDatabase,
}

pub fn run(request: DiffPrefixRequest<'_>) -> Result<DiffPrefixResponse, (DiffPrefixStateId, ErrorKind)> {
    let requested = RequestedPrefixDigestRange::parse(request.from_prefix_digest, request.to_prefix_digest)
        .map_err(|error| (DiffPrefixStateId::Setup, error))?;
    let sysroot = Sysroot::new(SysrootMode::ReadOnly).map_err(|error| (DiffPrefixStateId::Setup, error.into()))?;

    let mut context = Context::default();
    context.put(sysroot);
    context.put(requested);

    SequentialOrchestrator::new(stages![PreparingStage, ComparingStage]).run_unmutated(
        &mut context,
        request.base.cancel_token,
        &|event| report_progress(&request.base, event),
    )
}
