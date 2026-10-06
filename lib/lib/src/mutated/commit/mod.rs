// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::error::ErrorKind;
use upac_types::request::mutated::CommitRequest;
use upac_types::state::mutated::CommitStateId;

use upac_deploy::{Sysroot, SysrootMode};

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::transaction::TransactionStage;

mod transaction;

pub(crate) struct CommitInfo {
    pub subject: String,
    pub message: Option<String>,
}

pub fn run(request: CommitRequest<'_>) -> Result<(), (CommitStateId, ErrorKind)> {
    let sysroot = Sysroot::new(SysrootMode::ReadWrite).map_err(|error| (CommitStateId::Setup, error.into()))?;

    let mut context = Context::default();
    context.put(sysroot);
    context.put(CommitInfo {
        subject: request.subject.to_owned(),
        message: request.message.map(str::to_owned),
    });

    SequentialOrchestrator::new(stages![TransactionStage]).run_mutating(
        &mut context,
        request.base.cancel_token,
        &|event| request.base.report_progress(event),
    )
}
