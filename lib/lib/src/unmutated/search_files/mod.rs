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

use crate::orchestrator::{Context, Orchestrator, SequentialOrchestrator, run_unmutated};
use crate::search::Search;
use upac_types::SearchFileEntry;
use upac_types::states::SearchFilesStateId;

mod error;
mod searching;

pub struct SearchFilesData<'a> {
    pub search: &'a str,
    pub is_regex: bool,

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
            is_regex: request.is_regex,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
}

pub fn run(data: SearchFilesData) -> Result<(Vec<SearchFileEntry>,), (SearchFilesStateId, SearchFilesError)> {
    let search = Search::new(data.search, data.is_regex)
        .map_err(|error| (SearchFilesStateId::Setup, SearchFilesError::from(error)))?;

    let mut context = Context::new();
    context.put(search);
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let orchestrator = SequentialOrchestrator::new(vec![Box::new(SearchingStage)]);

    run_unmutated!(
        orchestrator,
        context,
        data.cancel_token,
        SearchFilesStateId,
        SearchFilesError,
        Vec<SearchFileEntry>
    )
}
