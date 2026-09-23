// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::TmpPath;
use upac_types::hook::Message;
use upac_types::request::mutated::RollbackRequest;
use upac_types::state::mutated::RollbackStateId;
use upac_types::traits::MessageHook;

use upac_macro::ContextValue;

use self::checkout::CheckoutStage;
use self::merge::MergeStage;
use self::swap::SwapStage;

use crate::deploy::retention::RetentionStage;
use crate::deploy::{Deploy, DeployMode};
use crate::orchestrator::context::Context;
use crate::orchestrator::{Orchestrator, SequentialOrchestrator, run_mutating, stages};
use crate::plugin::boot::BootPlugin;
use crate::scripts::HookStage;
use crate::scripts::pipeline::{Operation, PipelineTrigger};

pub use self::error::RollbackError;

mod checkout;
mod error;
mod merge;
mod swap;

#[derive(ContextValue)]
pub(crate) struct RequestedConfigDigest(pub String);

#[derive(ContextValue)]
pub(crate) struct TargetPrefixDigest(pub String);

#[derive(ContextValue)]
pub(crate) struct RequestedBootPlugin(pub String);

pub(crate) struct ResolvedBootEntry {
    pub plugin: BootPlugin,
    pub entry_name: String,
}

pub fn run(request: RollbackRequest<'_>) -> Result<(), (RollbackStateId, RollbackError)> {
    let deploy =
        Deploy::new(DeployMode::ReadOnly).map_err(|error| (RollbackStateId::Setup, RollbackError::from(error)))?;
    let cancel_token = unsafe { &*request.base.cancel_token };

    let mut context = Context::new();
    context.put(deploy);
    context.put(RequestedConfigDigest(request.config_digest.to_owned()));
    context.put(RequestedBootPlugin(request.boot_plugin.to_owned()));
    context.put(TmpPath(request.tmp_path.to_owned()));
    context.put(Box::new(Message::new(request.base.on_hook, request.base.hook_ctx)) as Box<dyn MessageHook>);

    let orchestrator = assemble();

    let result = run_mutating!(orchestrator, context, cancel_token, RollbackStateId, RollbackError);

    cancel_token.reset();

    result
}

fn assemble() -> SequentialOrchestrator<RollbackError> {
    SequentialOrchestrator::new(stages![
        HookStage {
            trigger: PipelineTrigger::pre(Operation::Rollback),
        },
        MergeStage,
        CheckoutStage,
        SwapStage,
        HookStage {
            trigger: PipelineTrigger::post(Operation::Rollback),
        },
        RetentionStage,
    ])
}
