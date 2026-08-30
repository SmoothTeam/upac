// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::request::CDiffPackagesRequest;

pub use self::error::DiffPackagesError;

use self::comparing::ComparingStage;
use self::preparing::PreparingStage;

use crate::orchestrator::{Context, Orchestrator, SequentialOrchestrator, run_unmutated};
use upac_types::states::DiffPackagesStateId;
use upac_types::{DiffPackageEntry, RequestedPrefixDigestRange};

mod comparing;
mod error;
mod preparing;

pub struct DiffPackagesData<'a> {
    pub from_prefix_digest: Option<&'a str>,
    pub to_prefix_digest: Option<&'a str>,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CDiffPackagesRequest> for DiffPackagesData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CDiffPackagesRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(DiffPackagesData {
            from_prefix_digest: (&request.from_prefix_digest).try_into()?,
            to_prefix_digest: (&request.to_prefix_digest).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
}

pub fn run(data: DiffPackagesData) -> Result<(Vec<DiffPackageEntry>,), (DiffPackagesStateId, DiffPackagesError)> {
    let mut context = Context::new();
    context.put(RequestedPrefixDigestRange {
        from: data.from_prefix_digest.map(str::to_owned),
        to: data.to_prefix_digest.map(str::to_owned),
    });
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let orchestrator = SequentialOrchestrator::new(vec![Box::new(PreparingStage), Box::new(ComparingStage)]);

    run_unmutated!(
        orchestrator,
        context,
        data.cancel_token,
        DiffPackagesStateId,
        DiffPackagesError,
        Vec<DiffPackageEntry>
    )
}
