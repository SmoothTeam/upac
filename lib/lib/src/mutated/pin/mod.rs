// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::request::CPinRequest;

pub use self::error::PinError;

use self::stage::SetPinnedStage;

use crate::deploy::{Deploy, DeployMode};
use crate::orchestrator::{Context, Orchestrator, SequentialOrchestrator, run_mutating};
use upac_types::states::PinStateId;

mod error;
mod stage;

pub(crate) struct RequestedPrefixDigest(pub String);
pub(crate) struct RequestedPinned(pub bool);

pub struct PinData<'a> {
    pub prefix_digest: &'a str,
    pub pinned: bool,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CPinRequest> for PinData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CPinRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(PinData {
            prefix_digest: (&request.prefix_digest).try_into()?,
            pinned: request.pinned,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
}

pub fn run(data: PinData) -> Result<(), (PinStateId, PinError)> {
    let deploy = Deploy::new(DeployMode::ReadWrite).map_err(|error| (PinStateId::Setup, PinError::from(error)))?;

    let mut context = Context::new();
    context.put(deploy);
    context.put(RequestedPrefixDigest(data.prefix_digest.to_owned()));
    context.put(RequestedPinned(data.pinned));
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let orchestrator = SequentialOrchestrator::new(vec![Box::new(SetPinnedStage)]);

    let result = run_mutating!(orchestrator, context, data.cancel_token, PinStateId, PinError);

    data.cancel_token.reset();

    result
}
