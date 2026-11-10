// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::BTreeMap;
use std::path::PathBuf;

use upac_types::diff::FileDiffKind;
use upac_types::error::ErrorKind;
use upac_types::request::unmutated::DiffConfigRequest;
use upac_types::response::unmutated::DiffConfigResponse;
use upac_types::state::unmutated::DiffConfigStateId;

use upac_database::MemoryDatabase;

use upac_deploy::{Sysroot, SysrootMode};

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::comparing::ComparingStage;
use self::preparing::PreparingStage;

use super::RequestedConfigDigestRange;

mod comparing;
mod preparing;

struct DiffConfigSnapshot {
    changed: BTreeMap<PathBuf, FileDiffKind>,
    from_database: MemoryDatabase,
    to_database: MemoryDatabase,
}

pub fn run(
    request: DiffConfigRequest<'_>,
) -> Result<DiffConfigResponse, (DiffConfigStateId, ErrorKind, Option<String>)> {
    let requested = RequestedConfigDigestRange::parse(request.from_config_digest, request.to_config_digest)
        .map_err(|error| (DiffConfigStateId::Setup, error, None))?;
    let sysroot =
        Sysroot::new(SysrootMode::ReadOnly).map_err(|error| (DiffConfigStateId::Setup, error.into(), None))?;

    let mut context = Context::default();
    context.put(sysroot);
    context.put(requested);

    SequentialOrchestrator::new(stages![PreparingStage, ComparingStage]).run_unmutated(
        &mut context,
        request.base.cancel_token,
        &|event| request.base.report_progress(event),
    )
}
