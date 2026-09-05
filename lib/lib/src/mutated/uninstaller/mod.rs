// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::VecDeque;
use std::os::raw::c_void;

use composefs::tree::FileSystem;

use uuid::Uuid;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::package::CPackageInfo;
use upac_abi::request::CUninstallRequest;

use crate::composefs::repository::ObjectID;
use crate::database::MemoryDatabase;
use crate::deploy::retention::RetentionStage;
use crate::deploy::{Deploy, DeployMode};
use crate::orchestrator::{Context, Orchestrator, SequentialOrchestrator, run_mutating};
use crate::plugin::boot::BootPlugin;
use crate::scripts::HookStage;
use crate::scripts::pipeline::{Operation, PipelineTrigger};

use upac_types::states::UninstallStateId;
use upac_types::{PackageEntry, Targets, TmpPath};

pub use self::error::UninstallError;

use self::checkout::CheckoutStage;
use self::commit::CommitTransactionStage;
use self::merge::MergeStage;
use self::open::OpenTransactionStage;
use self::preparation::PreparationStage;
use self::remove::RemovePackageStage;
use self::swap::SwapStage;

mod checkout;
mod commit;
mod error;
mod merge;
mod open;
mod preparation;
mod remove;
mod swap;

pub(crate) struct PackageUuidsToRemove(pub Vec<Uuid>);
pub(crate) struct NewPrefixDigest(pub String);
pub(crate) struct RemovedConfigPaths(pub Vec<String>);
pub(crate) struct Subject(pub String);
pub(crate) struct CommitMessage(pub Option<String>);
pub(crate) struct RequestedBootPlugin(pub Option<String>);
pub(crate) struct Purge(pub bool);
pub(crate) struct ResolvedBootEntry {
    pub plugin: BootPlugin,
    pub entry_name: String,
}

pub(crate) struct PendingUuids(pub VecDeque<Uuid>);
pub(crate) struct TotalPackages(pub u64);
pub(crate) struct WorkingTree(pub FileSystem<ObjectID>);
pub(crate) struct WorkingDatabase(pub MemoryDatabase);
pub(crate) struct WorkingRemovedConfigPaths(pub Vec<String>);

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
    pub boot_plugin: Option<&'a str>,
    pub purge: bool,

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
            boot_plugin: (&request.boot_plugin).try_into()?,
            purge: request.purge,

            tmp_path: (&request.tmp_path).try_into()?,

            subject: (&request.subject).try_into()?,
            message: (&request.message).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
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
    context.put(Subject(data.subject.to_owned()));
    context.put(CommitMessage(data.message.map(str::to_owned)));
    context.put(RequestedBootPlugin(data.boot_plugin.map(str::to_owned)));
    context.put(Purge(data.purge));
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

fn assemble() -> SequentialOrchestrator<UninstallError> {
    SequentialOrchestrator::new(vec![
        Box::new(HookStage {
            trigger: PipelineTrigger::pre(Operation::Uninstall),
        }),
        Box::new(PreparationStage),
        Box::new(OpenTransactionStage),
        Box::new(RemovePackageStage),
        Box::new(CommitTransactionStage),
        Box::new(MergeStage),
        Box::new(CheckoutStage),
        Box::new(SwapStage),
        Box::new(HookStage {
            trigger: PipelineTrigger::declarative(Operation::Uninstall),
        }),
        Box::new(HookStage {
            trigger: PipelineTrigger::post(Operation::Uninstall),
        }),
        Box::new(RetentionStage),
    ])
}
