// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::request::CListPackagesRequest;

pub use self::error::ListPackagesError;

use self::fetching::FetchingStage;

use crate::orchestrator::{Context, Orchestrator, SequentialOrchestrator, run_unmutated};
use crate::types::states::ListPackagesStateId;
use crate::types::{Branch, PackageMeta};

mod error;
mod fetching;

pub struct ListPackagesData<'a> {
    pub branch: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CListPackagesRequest> for ListPackagesData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CListPackagesRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(ListPackagesData {
            branch: (&request.base.branch).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
}

fn assemble() -> SequentialOrchestrator<ListPackagesError> {
    SequentialOrchestrator::new(vec![Box::new(FetchingStage)])
}

pub fn run(data: ListPackagesData) -> Result<(Vec<PackageMeta>,), (ListPackagesStateId, ListPackagesError)> {
    let mut context = Context::new();
    context.put(Branch(data.branch.to_owned()));
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let orchestrator = assemble();

    run_unmutated!(
        orchestrator,
        context,
        data.cancel_token,
        ListPackagesStateId,
        ListPackagesError,
        Vec<PackageMeta>
    )
}
