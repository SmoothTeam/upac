// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::request::CListCommitRequest;

pub use self::error::ListCommitError;

use self::fetching::FetchingStage;

use crate::orchestrator::{Context, Orchestrator, SequentialOrchestrator};
use crate::types::errors::CommonError;
use crate::types::states::ListCommitStateId;
use crate::types::{Branch, CommitEntry};

mod error;
mod fetching;

pub struct ListCommitData<'a> {
    pub prefix_digest: Option<&'a str>,

    pub branch: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CListCommitRequest> for ListCommitData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CListCommitRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(ListCommitData {
            prefix_digest: (&request.prefix_digest).try_into()?,

            branch: (&request.base.branch).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
}

fn assemble() -> SequentialOrchestrator<ListCommitError> {
    SequentialOrchestrator::new(vec![Box::new(FetchingStage)])
}

pub fn run(data: ListCommitData) -> Result<Vec<CommitEntry>, (ListCommitStateId, ListCommitError)> {
    let mut context = Context::new();
    context.put(Branch(data.branch.to_owned()));
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let orchestrator = assemble();

    orchestrator
        .run_concurrent(&mut context, data.cancel_token)
        .map_err(|(index, error)| (ListCommitStateId::from_stage_index(index), error))?;

    context.take::<Vec<CommitEntry>>().ok_or((
        ListCommitStateId::Setup,
        ListCommitError::from(CommonError::MissingResult),
    ))
}
