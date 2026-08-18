// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::os::raw::c_void;

use upac_abi::BootKind;
use upac_abi::FileDiffKind;
use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::package::CPackageInfo;
use upac_abi::request::CFilesRequest;

pub use self::error::FilesError;

use self::checkout::CheckoutStage;
use self::swap::SwapStage;
use self::transaction::TransactionStage;

use crate::orchestrator::{Context, Orchestrator, SequentialOrchestrator, run_mutating};
use crate::scripts::HookStage;
use crate::scripts::native::{NativeTrigger, Operation};
use upac_types::TmpPath;
use upac_types::states::FilesStateId;

mod checkout;
mod error;
mod swap;
mod transaction;

pub struct FilesData<'a> {
    #[expect(dead_code)]
    pub files: Vec<&'a str>,
    #[expect(dead_code)]
    pub file_kind: FileDiffKind,
    #[expect(dead_code)]
    pub file_package: &'a CPackageInfo,
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

impl<'a> TryFrom<&'a CFilesRequest> for FilesData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CFilesRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let file_package = unsafe { request.file_package.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;
        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(FilesData {
            files: Vec::try_from(&request.files)?,
            file_kind: request.file_kind,
            file_package,
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

pub fn run(data: FilesData) -> Result<(), (FilesStateId, FilesError)> {
    let mut context = Context::new();
    context.put(TmpPath(data.tmp_path.to_owned()));
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let orchestrator = assemble();

    let result = run_mutating!(orchestrator, context, data.cancel_token, FilesStateId, FilesError);

    data.cancel_token.reset();

    result
}

fn assemble() -> SequentialOrchestrator<FilesError> {
    SequentialOrchestrator::new(vec![
        Box::new(HookStage {
            trigger: NativeTrigger::pre(Operation::Files),
        }),
        Box::new(TransactionStage),
        Box::new(CheckoutStage),
        Box::new(SwapStage),
        Box::new(HookStage {
            trigger: NativeTrigger::post(Operation::Files),
        }),
    ])
}
