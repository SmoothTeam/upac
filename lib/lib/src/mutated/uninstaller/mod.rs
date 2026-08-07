// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::package::CPackageInfo;
use upac_abi::request::CUninstallRequest;

use crate::deploy::{Deploy, DeployMode};
use crate::orchestrator::{Context, Orchestrator, SequentialOrchestrator, run_mutating};
use crate::scripts::HookStage;
use crate::scripts::native::{NativeTrigger, Operation};
use crate::types::states::UninstallStateId;
use crate::types::{Branch, PackageEntry, Targets, TmpPath};

pub use self::error::UninstallError;

use self::boot_option::BootOptionStage;
use self::build::BuildStage;
use self::commit::CommitStage;
use self::config_merge::ConfigMergeStage;
use self::preparation::PreparationStage;
use self::prepare_boot::PrepareBootStage;

mod boot_option;
mod build;
mod commit;
mod config_merge;
mod error;
mod preparation;
mod prepare_boot;

pub struct UninstallPackage<'a> {
    pub name: &'a str,
    pub arch: &'a str,
    pub arch_sub: Option<&'a str>,
}

impl<'a> TryFrom<&'a CPackageInfo> for UninstallPackage<'a> {
    type Error = ErrorKind;

    fn try_from(info: &'a CPackageInfo) -> Result<Self, ErrorKind> {
        unsafe { info.validate()? };

        Ok(UninstallPackage {
            name: (&info.name).try_into()?,
            arch: (&info.arch).try_into()?,
            arch_sub: (&info.arch_sub).try_into()?,
        })
    }
}

pub struct UninstallData<'a> {
    pub packages: Vec<UninstallPackage<'a>>,

    pub branch: &'a str,

    pub tmp_path: &'a str,

    pub subject: &'a str,
    pub message: Option<&'a str>,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CUninstallRequest> for UninstallData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CUninstallRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(UninstallData {
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

fn assemble() -> SequentialOrchestrator<UninstallError> {
    SequentialOrchestrator::new(vec![
        Box::new(HookStage {
            trigger: NativeTrigger::pre(Operation::Uninstall),
        }),
        Box::new(PreparationStage),
        Box::new(BuildStage),
        Box::new(CommitStage),
        Box::new(ConfigMergeStage),
        Box::new(PrepareBootStage),
        Box::new(BootOptionStage),
        Box::new(HookStage {
            trigger: NativeTrigger::post(Operation::Uninstall),
        }),
    ])
}

pub fn run(data: UninstallData) -> Result<(), (UninstallStateId, UninstallError)> {
    let deploy =
        Deploy::new(DeployMode::ReadWrite).map_err(|error| (UninstallStateId::Setup, UninstallError::from(error)))?;

    let targets = Targets(
        data.packages
            .iter()
            .map(|package| PackageEntry {
                name: package.name.to_owned(),
                arch: package.arch.to_owned(),
                arch_sub: package.arch_sub.map(str::to_owned),
            })
            .collect(),
    );

    let mut context = Context::new();
    context.put(targets);
    context.put(deploy);
    context.put(TmpPath(data.tmp_path.to_owned()));
    context.put(Branch(data.branch.to_owned()));
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let orchestrator = assemble();

    let result = run_mutating!(
        orchestrator,
        context,
        data.cancel_token,
        UninstallStateId,
        UninstallError
    );

    data.cancel_token.reset();

    result
}
