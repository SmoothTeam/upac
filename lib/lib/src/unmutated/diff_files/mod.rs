// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::request::CDiffFilesRequest;

pub use self::error::DiffFilesError;

use self::comparing::ComparingStage;
use self::preparing::PreparingStage;

use crate::orchestrator::{Context, Orchestrator, SequentialOrchestrator};
use crate::types::errors::CommonError;
use crate::types::states::DiffFilesStateId;
use crate::types::{Branch, DiffFileEntry};

mod comparing;
mod error;
mod preparing;

pub struct DiffFilesData<'a> {
    pub from_commit_hash: Option<&'a str>,
    pub to_commit_hash: Option<&'a str>,

    pub branch: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CDiffFilesRequest> for DiffFilesData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CDiffFilesRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(DiffFilesData {
            from_commit_hash: (&request.from_commit_hash).try_into()?,
            to_commit_hash: (&request.to_commit_hash).try_into()?,

            branch: (&request.base.branch).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
}

fn assemble() -> SequentialOrchestrator<DiffFilesError> {
    SequentialOrchestrator::new(vec![Box::new(PreparingStage), Box::new(ComparingStage)])
}

pub fn run(data: DiffFilesData) -> Result<Vec<DiffFileEntry>, (DiffFilesStateId, DiffFilesError)> {
    let mut context = Context::new();
    context.put(Branch(data.branch.to_owned()));
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let orchestrator = assemble();

    orchestrator
        .run_concurrent(&mut context, data.cancel_token)
        .map_err(|(index, error)| (DiffFilesStateId::from_stage_index(index), error))?;

    context.take::<Vec<DiffFileEntry>>().ok_or((
        DiffFilesStateId::Setup,
        DiffFilesError::from(CommonError::MissingResult),
    ))
}
