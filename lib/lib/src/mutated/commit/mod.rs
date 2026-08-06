// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::request::CCommitRequest;

pub use self::error::CommitError;

use self::transaction::TransactionStage;

use crate::orchestrator::{Context, Orchestrator, SequentialOrchestrator, run_mutating};
use crate::scripts::HookStage;
use crate::scripts::native::{NativeTrigger, Operation, Timing};
use crate::types::states::CommitStateId;
use crate::types::{Branch, TmpPath};

mod error;
mod transaction;

pub struct CommitData<'a> {
    pub branch: &'a str,

    pub tmp_path: &'a str,

    pub subject: &'a str,
    pub message: Option<&'a str>,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CCommitRequest> for CommitData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CCommitRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(CommitData {
            branch: (&request.base.branch).try_into()?,

            tmp_path: (&request.tmp_path).try_into()?,

            subject: (&request.subject).try_into()?,
            message: (&request.message).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
}

fn assemble() -> SequentialOrchestrator<CommitError> {
    SequentialOrchestrator::new(vec![
        Box::new(HookStage {
            trigger: NativeTrigger {
                operation: Operation::Commit,
                timing: Timing::Pre,
            },
        }),
        Box::new(TransactionStage),
        Box::new(HookStage {
            trigger: NativeTrigger {
                operation: Operation::Commit,
                timing: Timing::Post,
            },
        }),
    ])
}

pub fn run(data: CommitData) -> Result<(), (CommitStateId, CommitError)> {
    let mut context = Context::new();
    context.put(TmpPath(data.tmp_path.to_owned()));
    context.put(Branch(data.branch.to_owned()));
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let orchestrator = assemble();

    let result = run_mutating!(orchestrator, context, data.cancel_token, CommitStateId, CommitError);

    data.cancel_token.reset();

    result
}
