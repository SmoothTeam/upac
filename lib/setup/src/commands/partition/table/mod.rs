// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::PathBuf;

use upac::errors::CommonError;
use upac::orchestrator::context::Context;
use upac::orchestrator::{Orchestrator, SequentialOrchestrator, run_mutating, stages};

use upac_types::request::partition::SetupPartitionTableRequest;
use upac_types::state::setup::PartitionTableStateId;

use upac_macro::ContextValue;

use self::wipe::WipeStage;
use self::write::WriteTableStage;

use crate::commands::partition::RequestedDevice;
use crate::commands::partition::error::PartitionError;

mod wipe;
mod write;

#[derive(ContextValue)]
pub(crate) struct ForceWipe(pub bool);

pub fn run(request: SetupPartitionTableRequest<'_>) -> Result<(), (PartitionTableStateId, PartitionError)> {
    let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or((
        PartitionTableStateId::Setup,
        PartitionError::from(CommonError::PipelineInvalid),
    ))?;

    let mut context = Context::new();
    context.put(request.base.message_hook());
    context.put(RequestedDevice(PathBuf::from(request.device_path)));
    context.put(ForceWipe(request.force_wipe));

    let orchestrator = SequentialOrchestrator::new(stages![WipeStage, WriteTableStage]);

    let result = run_mutating!(
        orchestrator,
        context,
        cancel_token,
        PartitionTableStateId,
        PartitionError
    );

    cancel_token.reset();

    result
}
