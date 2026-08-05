// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::request::CRollbackRequest;

pub use self::error::RollbackError;

use self::checkout::CheckoutStage;
use self::merge::MergeStage;
use self::swap::SwapStage;

use crate::orchestrator::error::OrchestratorError;
use crate::orchestrator::{Context, Orchestrator, SequentialOrchestrator};
use crate::scripts::HookStage;
use crate::scripts::native::{NativeTrigger, Operation, Timing};
use crate::types::states::RollbackStateId;
use crate::types::{Branch, TmpPath};

mod checkout;
mod error;
mod merge;
mod swap;

pub struct RollbackData<'a> {
    pub commit_hash: &'a str,

    pub branch: &'a str,

    pub tmp_path: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CRollbackRequest> for RollbackData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CRollbackRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(RollbackData {
            commit_hash: (&request.commit_hash).try_into()?,

            branch: (&request.base.branch).try_into()?,

            tmp_path: (&request.tmp_path).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
}

fn assemble() -> SequentialOrchestrator<RollbackError> {
    SequentialOrchestrator::new(vec![
        Box::new(HookStage {
            trigger: NativeTrigger {
                operation: Operation::Rollback,
                timing: Timing::Pre,
            },
        }),
        Box::new(MergeStage),
        Box::new(CheckoutStage),
        Box::new(SwapStage),
        Box::new(HookStage {
            trigger: NativeTrigger {
                operation: Operation::Rollback,
                timing: Timing::Post,
            },
        }),
    ])
}

pub fn run(data: RollbackData) -> Result<(), (RollbackStateId, RollbackError)> {
    let mut context = Context::new();
    context.put(TmpPath(data.tmp_path.to_owned()));
    context.put(Branch(data.branch.to_owned()));
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let orchestrator = assemble();

    let result = orchestrator
        .run_exclusive(&mut context, data.cancel_token)
        .map_err(|failure| match failure {
            OrchestratorError::Setup(lock_error) => (RollbackStateId::Setup, RollbackError::from(lock_error)),
            OrchestratorError::Stage(index, error) => (RollbackStateId::from_stage_index(index), error),
        });

    data.cancel_token.reset();

    result
}
