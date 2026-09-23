// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::VecDeque;

use upac_types::hook::Message;
use upac_types::request::mutated::GcRequest;
use upac_types::traits::MessageHook;

use upac_types::state::mutated::GcStateId;

use upac_macro::ContextValue;

use self::cleaning::CleaningStage;
use self::collect::CollectRootsStage;
use self::pruning::PruneStage;

use crate::deploy::{Deploy, DeployMode};
use crate::orchestrator::context::Context;
use crate::orchestrator::{Orchestrator, SequentialOrchestrator, run_mutating, stages};

pub use self::error::GcError;

mod cleaning;
mod collect;
mod error;
mod pruning;

pub(crate) struct DeployProgress {
    pub pending: VecDeque<String>,
    pub total: u64,
}

#[derive(ContextValue)]
pub(crate) struct CollectedRoots(pub Vec<String>);

pub fn run(request: GcRequest) -> Result<(), (GcStateId, GcError)> {
    let deploy = Deploy::new(DeployMode::ReadWrite).map_err(|error| (GcStateId::Setup, GcError::from(error)))?;
    let cancel_token = unsafe { &*request.base.cancel_token };

    let mut context = Context::new();
    context.put(deploy);
    context.put(Box::new(Message::new(request.base.on_hook, request.base.hook_ctx)) as Box<dyn MessageHook>);

    let orchestrator = SequentialOrchestrator::new(stages![PruneStage, CollectRootsStage, CleaningStage]);

    let result = run_mutating!(orchestrator, context, cancel_token, GcStateId, GcError);

    cancel_token.reset();

    result
}
