// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::request::CDiffFilesPrefixRequest;

pub use self::error::DiffFilesPrefixError;

use self::comparing::ComparingStage;
use self::preparing::PreparingStage;

use crate::orchestrator::{Context, Orchestrator, SequentialOrchestrator, run_unmutated};
use crate::types::DiffPrefixFileEntry;
use crate::types::states::DiffFilesPrefixStateId;

mod comparing;
mod error;
mod preparing;

pub struct DiffFilesPrefixData<'a> {
    pub from_prefix_digest: Option<&'a str>,
    pub to_prefix_digest: Option<&'a str>,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CDiffFilesPrefixRequest> for DiffFilesPrefixData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CDiffFilesPrefixRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(DiffFilesPrefixData {
            from_prefix_digest: (&request.from_prefix_digest).try_into()?,
            to_prefix_digest: (&request.to_prefix_digest).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
}

pub fn run(
    data: DiffFilesPrefixData,
) -> Result<(Vec<DiffPrefixFileEntry>,), (DiffFilesPrefixStateId, DiffFilesPrefixError)> {
    let mut context = Context::new();
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let orchestrator = SequentialOrchestrator::new(vec![Box::new(PreparingStage), Box::new(ComparingStage)]);

    run_unmutated!(
        orchestrator,
        context,
        data.cancel_token,
        DiffFilesPrefixStateId,
        DiffFilesPrefixError,
        Vec<DiffPrefixFileEntry>
    )
}
