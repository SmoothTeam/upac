// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::request::CSearchMetaRequest;

pub use self::error::SearchMetaError;

use self::searching::SearchingStage;

use crate::orchestrator::{Context, Orchestrator, SequentialOrchestrator, run_unmutated};
use crate::search::Search;
use upac_types::PackageMeta;
use upac_types::states::SearchMetaStateId;

mod error;
mod searching;

pub struct SearchMetaData<'a> {
    pub search: &'a str,
    pub is_regex: bool,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CSearchMetaRequest> for SearchMetaData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CSearchMetaRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(SearchMetaData {
            search: (&request.search).try_into()?,
            is_regex: request.is_regex,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
}

pub fn run(data: SearchMetaData) -> Result<(Vec<PackageMeta>,), (SearchMetaStateId, SearchMetaError)> {
    let search = Search::new(data.search, data.is_regex)
        .map_err(|error| (SearchMetaStateId::Setup, SearchMetaError::from(error)))?;

    let mut context = Context::new();
    context.put(search);
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let orchestrator = SequentialOrchestrator::new(vec![Box::new(SearchingStage)]);

    run_unmutated!(
        orchestrator,
        context,
        data.cancel_token,
        SearchMetaStateId,
        SearchMetaError,
        Vec<PackageMeta>
    )
}
