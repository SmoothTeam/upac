// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::request::CSearchFilesRequest;

pub use self::error::SearchFilesError;

use self::searching::SearchingStage;

use crate::orchestrator::{Context, Orchestrator};
use crate::types::errors::CommonError;
use crate::types::states::SearchFilesStateId;
use crate::types::{Branch, SearchFileEntry};

mod error;
mod searching;

pub struct SearchFilesData<'a> {
    pub search: &'a str,

    pub branch: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CSearchFilesRequest> for SearchFilesData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CSearchFilesRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(SearchFilesData {
            search: (&request.search).try_into()?,

            branch: (&request.base.branch).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
}

fn assemble() -> Orchestrator<SearchFilesError> {
    Orchestrator::new(vec![Box::new(SearchingStage)])
}

pub fn run(data: SearchFilesData) -> Result<Vec<SearchFileEntry>, (SearchFilesStateId, SearchFilesError)> {
    let mut context = Context::new();
    context.put(Branch(data.branch.to_owned()));
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let mut orchestrator = assemble();

    orchestrator
        .run_concurrent(&mut context, data.cancel_token)
        .map_err(|(index, error)| (SearchFilesStateId::from_stage_index(index), error))?;

    context.take::<Vec<SearchFileEntry>>().ok_or((
        SearchFilesStateId::Setup,
        SearchFilesError::from(CommonError::MissingResult),
    ))
}
