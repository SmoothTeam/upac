// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::request::CInstallRequest;

pub use self::error::InstallError;

use self::checkout::CheckoutStage;
use self::merge::MergeStage;
use self::preparation::PreparationStage;
use self::swap::SwapStage;
use self::transaction::TransactionStage;

use crate::orchestrator::error::OrchestratorError;
use crate::orchestrator::{Context, Orchestrator, SequentialOrchestrator};
use crate::types::states::InstallStateId;
use crate::types::{Branch, PackageTemp, TmpPath};

mod checkout;
mod error;
mod merge;
mod preparation;
mod swap;
mod transaction;

pub struct InstallData<'a> {
    pub packages: Vec<PackageTemp>,

    pub branch: &'a str,

    pub tmp_path: &'a str,

    pub subject: &'a str,
    pub message: Option<&'a str>,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CInstallRequest> for InstallData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CInstallRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(InstallData {
            packages: Vec::try_from(&request.packages)?,

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

fn assemble() -> SequentialOrchestrator<InstallError> {
    SequentialOrchestrator::new(vec![
        Box::new(PreparationStage),
        Box::new(TransactionStage),
        Box::new(MergeStage),
        Box::new(CheckoutStage),
        Box::new(SwapStage),
    ])
}

pub fn run(data: InstallData) -> Result<(), (InstallStateId, InstallError)> {
    let mut context = Context::new();
    context.put(data.packages);
    context.put(TmpPath(data.tmp_path.to_owned()));
    context.put(Branch(data.branch.to_owned()));
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let orchestrator = assemble();

    let result = orchestrator
        .run_exclusive(&mut context, data.cancel_token)
        .map_err(|failure| match failure {
            OrchestratorError::Setup(lock_error) => (InstallStateId::Setup, InstallError::from(lock_error)),
            OrchestratorError::Stage(index, error) => (InstallStateId::from_stage_index(index), error),
        });

    data.cancel_token.reset();

    result
}
