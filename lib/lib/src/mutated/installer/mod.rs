// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::VecDeque;
use std::os::raw::c_void;

use composefs::tree::FileSystem;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::request::CInstallRequest;

use upac_types::{DeclarativeTrigger, PackageTemp};

pub use self::error::InstallError;

use self::checkout::CheckoutStage;
use self::commit::CommitTransactionStage;
use self::fetching::FetchingStage;
use self::import::ImportPackageStage;
use self::merge::MergeStage;
use self::open::OpenTransactionStage;
use self::preparation::PreparationStage;
use self::swap::SwapStage;

use crate::composefs::repository::ObjectID;
use crate::database::MemoryDatabase;
use crate::deploy::retention::RetentionStage;
use crate::deploy::{Deploy, DeployMode};
use crate::errors::CommonError;
use crate::orchestrator::{Context, Orchestrator, SequentialOrchestrator, run_mutating};
use crate::plugin::boot::BootPlugin;
use crate::plugin::decoder::unpack::PackageUnpacker;
use crate::scripts::HookStage;
use crate::scripts::pipeline::{Operation, PipelineTrigger};
use upac_types::TmpPath;
use upac_types::states::InstallStateId;

mod checkout;
mod commit;
mod error;
mod fetching;
mod import;
mod merge;
mod open;
mod preparation;
mod swap;

pub(crate) struct NewPrefixDigest(pub String);
pub(crate) struct NewConfigDefaults(pub FileSystem<ObjectID>);
pub(crate) struct Subject(pub String);
pub(crate) struct CommitMessage(pub Option<String>);
pub(crate) struct RequestedBootPlugin(pub Option<String>);
pub(crate) struct AllowConflictFiles(pub bool);
pub(crate) struct ResolvedBootEntry {
    pub plugin: BootPlugin,
    pub entry_name: String,
}

pub(crate) struct PendingPackagePaths(pub VecDeque<String>);
pub(crate) struct UnpackerState(pub PackageUnpacker);
pub(crate) struct PendingPackages(pub VecDeque<(PackageTemp, DeclarativeTrigger)>);
pub(crate) struct TotalPackages(pub u64);
pub(crate) struct ImportedTree(pub FileSystem<ObjectID>);
pub(crate) struct ImportedConfigDefaults(pub FileSystem<ObjectID>);
pub(crate) struct ImportedDatabase(pub MemoryDatabase);

pub struct InstallData<'a> {
    pub packages: Vec<&'a str>,
    pub boot_plugin: Option<&'a str>,
    pub allow_conflict_files: bool,

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
            boot_plugin: (&request.boot_plugin).try_into()?,
            allow_conflict_files: request.allow_conflict_files,

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
    let deploy =
        Deploy::new(DeployMode::ReadWrite).map_err(|error| (InstallStateId::Setup, InstallError::from(error)))?;
    let unpacker = PackageUnpacker::new()
        .map_err(|error| (InstallStateId::Setup, InstallError::from(CommonError::Decoder(error))))?;

    let total_packages = data.packages.len() as u64;

    let mut context = Context::new();
    context.put(deploy);
    context.put(UnpackerState(unpacker));
    context.put(PendingPackagePaths(
        data.packages.iter().map(|path| (*path).to_owned()).collect(),
    ));
    context.put(PendingPackages(VecDeque::new()));
    context.put(TotalPackages(total_packages));
    context.put(TmpPath(data.tmp_path.to_owned()));
    context.put(Subject(data.subject.to_owned()));
    context.put(CommitMessage(data.message.map(str::to_owned)));
    context.put(RequestedBootPlugin(data.boot_plugin.map(str::to_owned)));
    context.put(AllowConflictFiles(data.allow_conflict_files));
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let orchestrator = assemble();

    let result = run_mutating!(orchestrator, context, data.cancel_token, InstallStateId, InstallError);

    data.cancel_token.reset();

    result
}

fn assemble() -> SequentialOrchestrator<InstallError> {
    SequentialOrchestrator::new(vec![
        Box::new(HookStage {
            trigger: PipelineTrigger::pre(Operation::Install),
        }),
        Box::new(FetchingStage),
        Box::new(PreparationStage),
        Box::new(OpenTransactionStage),
        Box::new(ImportPackageStage),
        Box::new(CommitTransactionStage),
        Box::new(MergeStage),
        Box::new(CheckoutStage),
        Box::new(SwapStage),
        Box::new(HookStage {
            trigger: PipelineTrigger::declarative(Operation::Install),
        }),
        Box::new(HookStage {
            trigger: PipelineTrigger::post(Operation::Install),
        }),
        Box::new(RetentionStage),
    ])
}
