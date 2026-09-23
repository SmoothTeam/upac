// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::VecDeque;

use upac_types::hook::Message;
use upac_types::request::mutated::MimeSyncRequest;
use upac_types::state::mutated::MimeStateId;
use upac_types::traits::MessageHook;

use upac_macro::ContextValue;

use self::preparing::PreparingStage;
use self::rendering::RenderingStage;
use self::writing::WritingStage;

use crate::orchestrator::context::Context;
use crate::orchestrator::{Orchestrator, SequentialOrchestrator, run_mutating, stages};

pub use self::error::MimeError;

mod error;
mod preparing;
mod rendering;
mod writing;

#[derive(ContextValue)]
pub(crate) struct DesktopContent(pub String);

pub(crate) struct WriteProgress {
    pub pending: VecDeque<(&'static str, String)>,
    pub total: u64,
}

pub fn run(request: MimeSyncRequest) -> Result<(), (MimeStateId, MimeError)> {
    let cancel_token = unsafe { &*request.base.cancel_token };

    let mut context = Context::new();
    context.put(Box::new(Message::new(request.base.on_hook, request.base.hook_ctx)) as Box<dyn MessageHook>);

    let orchestrator = SequentialOrchestrator::new(stages![PreparingStage, RenderingStage, WritingStage]);

    let result = run_mutating!(orchestrator, context, cancel_token, MimeStateId, MimeError);

    cancel_token.reset();

    result
}
