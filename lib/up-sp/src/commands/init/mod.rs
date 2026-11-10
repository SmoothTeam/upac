// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::PathBuf;

use upac_types::error::ErrorKind;
use upac_types::request::partition::SetupPartitionTableRequest;
use upac_types::state::setup::PartitionTableStateId;

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::wipe::WipeStage;
use self::write::WriteTableStage;

mod wipe;
mod write;

pub(crate) struct RequestedDevice(pub PathBuf);

pub(crate) struct ForceWipe(pub bool);

pub fn run(request: SetupPartitionTableRequest<'_>) -> Result<(), (PartitionTableStateId, ErrorKind, Option<String>)> {
    let mut context = Context::default();
    context.put(RequestedDevice(PathBuf::from(request.device_path)));
    context.put(ForceWipe(request.force_wipe));

    SequentialOrchestrator::new(stages![WipeStage, WriteTableStage]).run_mutating(
        &mut context,
        request.base.cancel_token,
        &|event| request.base.report_progress(event),
    )
}
