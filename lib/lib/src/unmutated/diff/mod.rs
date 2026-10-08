// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::PathBuf;

use upac_types::diff::{DiffFileSource, FileDiffKind};
use upac_types::error::ErrorKind;
use upac_types::package::PackageMeta;
use upac_types::request::unmutated::DiffRequest;
use upac_types::response::unmutated::DiffResponse;
use upac_types::state::unmutated::DiffStateId;

use upac_database::MemoryDatabase;

use upac_deploy::{Sysroot, SysrootMode};

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::comparing::ComparingStage;
use self::preparing::PreparingStage;

use super::{RequestedConfigDigestRange, RequestedPrefixDigestRange};

mod comparing;
mod preparing;

struct DiffSnapshot {
    from_packages: Vec<PackageMeta>,
    to_packages: Vec<PackageMeta>,
    changed_files: Vec<(PathBuf, FileDiffKind, DiffFileSource)>,
    from_database: MemoryDatabase,
    to_database: MemoryDatabase,
}

pub fn run(request: DiffRequest<'_>) -> Result<DiffResponse, (DiffStateId, ErrorKind, Option<String>)> {
    let requested_prefixes = RequestedPrefixDigestRange::parse(request.from_prefix_digest, request.to_prefix_digest)
        .map_err(|error| (DiffStateId::Setup, error, None))?;
    let requested_configs = RequestedConfigDigestRange::parse(request.from_config_digest, request.to_config_digest)
        .map_err(|error| (DiffStateId::Setup, error, None))?;
    let sysroot = Sysroot::new(SysrootMode::ReadOnly).map_err(|error| (DiffStateId::Setup, error.into(), None))?;

    let mut context = Context::default();
    context.put(sysroot);
    context.put(requested_prefixes);
    context.put(requested_configs);

    SequentialOrchestrator::new(stages![PreparingStage, ComparingStage]).run_unmutated(
        &mut context,
        request.base.cancel_token,
        &|event| request.base.report_progress(event),
    )
}
