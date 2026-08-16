// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::request::CGcRequest;

pub use self::error::GcError;

use self::cleaning::CleaningStage;

use crate::orchestrator::{Context, Orchestrator, SequentialOrchestrator, run_mutating};
use upac_types::states::GcStateId;

mod cleaning;
mod error;

pub struct GcData<'a> {
    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CGcRequest> for GcData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CGcRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(GcData {
            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
}

pub fn run(data: GcData) -> Result<(), (GcStateId, GcError)> {
    let mut context = Context::new();
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let orchestrator = SequentialOrchestrator::new(vec![Box::new(CleaningStage)]);

    let result = run_mutating!(orchestrator, context, data.cancel_token, GcStateId, GcError);

    data.cancel_token.reset();

    result
}
