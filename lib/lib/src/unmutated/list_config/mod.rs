// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::request::CListConfigRequest;

pub use self::error::ListConfigError;

use self::fetching::FetchingStage;

use crate::orchestrator::{Context, Orchestrator, SequentialOrchestrator, run_unmutated};
use upac_types::states::ListConfigStateId;
use upac_types::{ConfigCommitEntry, RequestedPrefixDigest};

mod error;
mod fetching;

pub struct ListConfigData<'a> {
    pub prefix_digest: Option<&'a str>,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CListConfigRequest> for ListConfigData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CListConfigRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(ListConfigData {
            prefix_digest: (&request.prefix_digest).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
}

pub fn run(data: ListConfigData) -> Result<(Vec<ConfigCommitEntry>,), (ListConfigStateId, ListConfigError)> {
    let mut context = Context::new();
    context.put(RequestedPrefixDigest(data.prefix_digest.map(str::to_owned)));
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let orchestrator = SequentialOrchestrator::new(vec![Box::new(FetchingStage)]);

    run_unmutated!(
        orchestrator,
        context,
        data.cancel_token,
        ListConfigStateId,
        ListConfigError,
        Vec<ConfigCommitEntry>
    )
}
