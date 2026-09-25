// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::request::mutated::PinRequest;

use upac_types::state::mutated::PinStateId;

use upac_macro::ContextValue;

use upac_deploy::{Deploy, DeployMode};

use upac_orchestrator::context::Context;
use upac_orchestrator::{Orchestrator, SequentialOrchestrator, run_mutating, stages};

use self::stage::SetPinnedStage;

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
    context.put(request.base.message_hook());

    let orchestrator = SequentialOrchestrator::new(stages![SetPinnedStage]);

    let result = run_mutating!(orchestrator, context, cancel_token, PinStateId, PinError);

    cancel_token.reset();

    result
}
