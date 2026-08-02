// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::request::CDiffRequest;

pub use self::error::DiffError;

use self::comparing::ComparingStage;
use self::preparing::PreparingStage;

use crate::orchestrator::{Context, Orchestrator};
use crate::types::errors::CommonError;
use crate::types::states::DiffStateId;
use crate::types::{Branch, DiffFileEntry, DiffPackageEntry};

mod comparing;
mod error;
mod preparing;

pub struct DiffData<'a> {
    pub from_commit_hash: Option<&'a str>,
    pub to_commit_hash: Option<&'a str>,

    pub branch: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CDiffRequest> for DiffData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CDiffRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(DiffData {
            from_commit_hash: (&request.from_commit_hash).try_into()?,
            to_commit_hash: (&request.to_commit_hash).try_into()?,

            branch: (&request.base.branch).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
}

fn assemble() -> Orchestrator<DiffError> {
    Orchestrator::new(vec![Box::new(PreparingStage), Box::new(ComparingStage)])
}

pub fn run(data: DiffData) -> Result<(Vec<DiffFileEntry>, Vec<DiffPackageEntry>), (DiffStateId, DiffError)> {
    let mut context = Context::new();
    context.put(Branch(data.branch.to_owned()));
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let mut orchestrator = assemble();

    orchestrator
        .run_concurrent(&mut context, data.cancel_token)
        .map_err(|(index, error)| (DiffStateId::from_stage_index(index), error))?;

    let files = context
        .take::<Vec<DiffFileEntry>>()
        .ok_or((DiffStateId::Setup, DiffError::from(CommonError::MissingResult)))?;
    let diff_packages = context
        .take::<Vec<DiffPackageEntry>>()
        .ok_or((DiffStateId::Setup, DiffError::from(CommonError::MissingResult)))?;

    Ok((files, diff_packages))
}
