// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::PathBuf;

use upac_types::error::ErrorKind;
use upac_types::request::partition::{PartitionKind, SetupPartitionAddRequest};
use upac_types::response::partition::SetupPartitionAddResponse;
use upac_types::state::setup::PartitionAddStateId;

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::insert::InsertEntryStage;
use self::settle::SettleStage;

mod insert;
mod settle;

pub(crate) struct RequestedDevice(pub PathBuf);

pub(crate) struct RequestedPartition {
    pub label: String,
    pub size_mib: u64,
    pub kind: PartitionKind,
}

pub fn run(
    request: SetupPartitionAddRequest<'_>,
) -> Result<SetupPartitionAddResponse, (PartitionAddStateId, ErrorKind)> {
    let mut context = Context::default();
    context.put(RequestedDevice(PathBuf::from(request.device_path)));
    context.put(RequestedPartition {
        label: request.label.to_owned(),
        size_mib: request.size_mib,
        kind: request.kind,
    });

    SequentialOrchestrator::new(stages![InsertEntryStage, SettleStage]).run_mutating_with_response(
        &mut context,
        request.base.cancel_token,
        &|event| request.base.report_progress(event),
    )
}
