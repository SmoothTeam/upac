// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::hook::Message;
use upac_types::request::mutated::PinRequest;
use upac_types::traits::MessageHook;

use upac_types::state::mutated::PinStateId;

use upac_macro::ContextValue;

use self::stage::SetPinnedStage;

use crate::deploy::{Deploy, DeployMode};
use crate::orchestrator::context::Context;
use crate::orchestrator::{Orchestrator, SequentialOrchestrator, run_mutating, stages};

pub use self::error::PinError;

mod error;
mod stage;

#[derive(ContextValue)]
pub(crate) struct RequestedPrefixDigest(pub String);

#[derive(ContextValue)]
pub(crate) struct RequestedPinned(pub bool);

pub fn run(request: PinRequest<'_>) -> Result<(), (PinStateId, PinError)> {
    let deploy = Deploy::new(DeployMode::ReadWrite).map_err(|error| (PinStateId::Setup, PinError::from(error)))?;
    let cancel_token = unsafe { &*request.base.cancel_token };

    let mut context = Context::new();
    context.put(deploy);
    context.put(RequestedPrefixDigest(request.prefix_digest.to_owned()));
    context.put(RequestedPinned(request.pinned));
    context.put(Box::new(Message::new(request.base.on_hook, request.base.hook_ctx)) as Box<dyn MessageHook>);

    let orchestrator = SequentialOrchestrator::new(stages![SetPinnedStage]);

    let result = run_mutating!(orchestrator, context, cancel_token, PinStateId, PinError);

    cancel_token.reset();

    result
}
