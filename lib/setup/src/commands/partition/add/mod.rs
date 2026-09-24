// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::PathBuf;

use upac::errors::CommonError;
use upac::orchestrator::context::Context;
use upac::orchestrator::{Orchestrator, SequentialOrchestrator, run_mutating, stages};

use upac_abi::PartitionKind;

use upac_types::request::partition::SetupPartitionAddRequest;
use upac_types::response::partition::SetupPartitionAddResponse;
use upac_types::state::setup::PartitionAddStateId;

use upac_macro::ContextValue;

use self::insert::InsertEntryStage;
use self::settle::SettleStage;

use crate::commands::partition::RequestedDevice;
use crate::commands::partition::error::PartitionError;

mod insert;
mod settle;

#[derive(ContextValue)]
pub(crate) struct InsertedPartitionNumber(pub u32);

pub(crate) struct RequestedPartition {
    pub label: String,
    pub size_mib: u64,
    pub kind: PartitionKind,
}

pub fn run(
    request: SetupPartitionAddRequest<'_>,
) -> Result<SetupPartitionAddResponse, (PartitionAddStateId, PartitionError)> {
    let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or((
        PartitionAddStateId::Setup,
        PartitionError::from(CommonError::PipelineInvalid),
    ))?;

    let mut context = Context::new();
    context.put(request.base.message_hook());
    context.put(RequestedDevice(PathBuf::from(request.device_path)));
    context.put(RequestedPartition {
        label: request.label.to_owned(),
        size_mib: request.size_mib,
        kind: request.kind,
    });

    let orchestrator = SequentialOrchestrator::new(stages![InsertEntryStage, SettleStage]);

    let result = run_mutating!(orchestrator, context, cancel_token, PartitionAddStateId, PartitionError);

    cancel_token.reset();

    result?;

    context.take::<SetupPartitionAddResponse>().ok_or((
        PartitionAddStateId::Setup,
        PartitionError::from(CommonError::MissingResult),
    ))
}
