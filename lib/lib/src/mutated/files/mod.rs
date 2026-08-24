// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::package::CPackageInfo;
use upac_abi::request::CFilesRequest;
use upac_abi::{DiffFileSource, FileDiffKind};

pub use self::error::FilesError;

use self::checkout::CheckoutStage;
use self::swap::SwapStage;
use self::transaction::TransactionStage;

use crate::deploy::{Deploy, DeployMode};
use crate::orchestrator::{Context, Orchestrator, SequentialOrchestrator, run_mutating};
use crate::plugin::boot::BootPlugin;
use crate::scripts::HookStage;
use crate::scripts::native::{NativeTrigger, Operation};
use upac_types::TmpPath;
use upac_types::states::FilesStateId;

mod checkout;
mod error;
mod swap;
mod transaction;

pub(crate) struct RequestedFileKind(pub FileDiffKind);
pub(crate) struct RequestedFileScope(pub DiffFileSource);
pub(crate) struct RequestedFilePackage {
    pub name: String,
    pub arch: String,
    pub arch_sub: Option<String>,
}
pub(crate) struct NewPrefixDigest(pub String);
pub(crate) struct Subject(pub String);
pub(crate) struct CommitMessage(pub Option<String>);
pub(crate) struct RequestedBootPlugin(pub Option<String>);
pub(crate) struct ResolvedBootEntry {
    pub plugin: BootPlugin,
    pub entry_name: String,
}

pub struct FilesPackage<'a> {
    pub name: &'a str,
    pub arch: &'a str,
    pub arch_sub: Option<&'a str>,
}

impl<'a> TryFrom<&'a CPackageInfo> for FilesPackage<'a> {
    type Error = ErrorKind;

    fn try_from(info: &'a CPackageInfo) -> Result<Self, ErrorKind> {
        unsafe { info.validate()? };

        Ok(FilesPackage {
            name: (&info.name).try_into()?,
            arch: (&info.arch).try_into()?,
            arch_sub: (&info.arch_sub).try_into()?,
        })
    }
}

pub struct FilesData<'a> {
    pub files: Vec<&'a str>,
    pub file_kind: FileDiffKind,
    pub scope: DiffFileSource,
    pub file_package: FilesPackage<'a>,
    pub boot_plugin: Option<&'a str>,

    pub tmp_path: &'a str,

    pub subject: &'a str,
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
            scope: request.scope,
            file_package: FilesPackage::try_from(file_package)?,
            boot_plugin: (&request.boot_plugin).try_into()?,

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
    let deploy = Deploy::new(DeployMode::ReadWrite).map_err(|error| (FilesStateId::Setup, FilesError::from(error)))?;

    let mut context = Context::new();
    context.put(deploy);
    context.put(
        data.files
            .iter()
            .map(|path| (*path).to_owned())
            .collect::<Vec<String>>(),
    );
    context.put(RequestedFileKind(data.file_kind));
    context.put(RequestedFileScope(data.scope));
    context.put(RequestedFilePackage {
        name: data.file_package.name.to_owned(),
        arch: data.file_package.arch.to_owned(),
        arch_sub: data.file_package.arch_sub.map(str::to_owned),
    });
    context.put(TmpPath(data.tmp_path.to_owned()));
    context.put(Subject(data.subject.to_owned()));
    context.put(CommitMessage(data.message.map(str::to_owned)));
    context.put(RequestedBootPlugin(data.boot_plugin.map(str::to_owned)));
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
