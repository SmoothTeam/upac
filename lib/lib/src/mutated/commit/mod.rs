// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::TmpPath;
use upac_types::hook::Message;
use upac_types::request::mutated::CommitRequest;
use upac_types::state::mutated::CommitStateId;
use upac_types::traits::MessageHook;

use self::transaction::TransactionStage;

use crate::deploy::retention::RetentionStage;
use crate::deploy::{Deploy, DeployMode};
use crate::orchestrator::context::Context;
use crate::orchestrator::{Orchestrator, SequentialOrchestrator, run_mutating, stages};
use crate::scripts::HookStage;
use crate::scripts::pipeline::{Operation, PipelineTrigger};

pub use self::error::CommitError;

mod error;
mod transaction;

pub(crate) struct CommitInfo {
    pub subject: String,
    pub message: Option<String>,
}

pub fn run(request: CommitRequest<'_>) -> Result<(), (CommitStateId, CommitError)> {
    let deploy =
        Deploy::new(DeployMode::ReadWrite).map_err(|error| (CommitStateId::Setup, CommitError::from(error)))?;
    let cancel_token = unsafe { &*request.base.cancel_token };

    let mut context = Context::new();
    context.put(deploy);
    context.put(TmpPath(request.tmp_path.to_owned()));
    context.put(CommitInfo {
        subject: request.subject.to_owned(),
        message: request.message.map(str::to_owned),
    });
    context.put(Box::new(Message::new(request.base.on_hook, request.base.hook_ctx)) as Box<dyn MessageHook>);

    let orchestrator = assemble();

    let result = run_mutating!(orchestrator, context, cancel_token, CommitStateId, CommitError);

    cancel_token.reset();

    result
}

fn assemble() -> SequentialOrchestrator<CommitError> {
    SequentialOrchestrator::new(stages![
        HookStage {
            trigger: PipelineTrigger::pre(Operation::Commit),
        },
        TransactionStage,
        HookStage {
            trigger: PipelineTrigger::post(Operation::Commit),
        },
        RetentionStage,
    ])
}
