// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::os::raw::c_void;

use upac_abi::BootKind;
use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::request::CInstallRequest;

pub use self::error::InstallError;

use self::checkout::CheckoutStage;
use self::merge::MergeStage;
use self::preparation::PreparationStage;
use self::swap::SwapStage;
use self::transaction::TransactionStage;

use crate::orchestrator::{Context, Orchestrator, SequentialOrchestrator, run_mutating};
use crate::scripts::HookStage;
use crate::scripts::native::{NativeTrigger, Operation};
use upac_types::TmpPath;
use upac_types::states::InstallStateId;

mod checkout;
mod error;
mod merge;
mod preparation;
mod swap;
mod transaction;

pub struct InstallData<'a> {
    pub packages: Vec<&'a str>,
    #[expect(dead_code)]
    pub boot_kind: BootKind,

    pub tmp_path: &'a str,

    #[expect(dead_code)]
    pub subject: &'a str,
    #[expect(dead_code)]
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
            boot_kind: request.boot_kind,

            tmp_path: (&request.tmp_path).try_into()?,

            subject: (&request.subject).try_into()?,
            message: (&request.message).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
}

pub fn run(data: InstallData) -> Result<(), (InstallStateId, InstallError)> {
    let mut context = Context::new();
    context.put(
        data.packages
            .iter()
            .map(|path| (*path).to_owned())
            .collect::<Vec<String>>(),
    );
    context.put(TmpPath(data.tmp_path.to_owned()));
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let orchestrator = assemble();

    let result = run_mutating!(orchestrator, context, data.cancel_token, InstallStateId, InstallError);

    data.cancel_token.reset();

    result
}

fn assemble() -> SequentialOrchestrator<InstallError> {
    SequentialOrchestrator::new(vec![
        Box::new(HookStage {
            trigger: NativeTrigger::pre(Operation::Install),
        }),
        Box::new(PreparationStage),
        Box::new(TransactionStage),
        Box::new(MergeStage),
        Box::new(CheckoutStage),
        Box::new(SwapStage),
        Box::new(HookStage {
            trigger: NativeTrigger::post(Operation::Install),
        }),
    ])
}
