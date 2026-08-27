// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::os::raw::c_void;

use composefs::tree::FileSystem;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::request::CUpdateRequest;

pub use self::error::UpdateError;

use self::checkout::CheckoutStage;
use self::fetching::FetchingStage;
use self::merge::MergeStage;
use self::preparation::PreparationStage;
use self::swap::SwapStage;
use self::transaction::TransactionStage;

use crate::composefs::repository::ObjectID;
use crate::deploy::retention::RetentionStage;
use crate::deploy::{Deploy, DeployMode};
use crate::orchestrator::{Context, Orchestrator, SequentialOrchestrator, run_mutating};
use crate::plugin::boot::BootPlugin;
use crate::scripts::HookStage;
use crate::scripts::pipeline::{Operation, PipelineTrigger};
use upac_types::TmpPath;
use upac_types::states::UpdateStateId;

mod checkout;
mod error;
mod fetching;
mod merge;
mod preparation;
mod swap;
mod transaction;

pub(crate) struct NewPrefixDigest(pub String);
pub(crate) struct NewConfigDefaults(pub FileSystem<ObjectID>);
pub(crate) struct RemovedConfigPaths(pub Vec<String>);
pub(crate) struct Subject(pub String);
pub(crate) struct CommitMessage(pub Option<String>);
pub(crate) struct RequestedBootPlugin(pub Option<String>);
pub(crate) struct ResolvedBootEntry {
    pub plugin: BootPlugin,
    pub entry_name: String,
}
pub(crate) struct AllowDowngrade(pub bool);
pub(crate) struct AllowConflictFiles(pub bool);

pub struct UpdateData<'a> {
    pub packages: Vec<&'a str>,
    pub boot_plugin: Option<&'a str>,
    pub allow_downgrade: bool,
    pub allow_conflict_files: bool,

    pub tmp_path: &'a str,

    pub subject: &'a str,
    pub message: Option<&'a str>,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CUpdateRequest> for UpdateData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CUpdateRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(UpdateData {
            packages: Vec::try_from(&request.packages)?,
            boot_plugin: (&request.boot_plugin).try_into()?,
            allow_downgrade: request.allow_downgrade,
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

pub fn run(data: UpdateData) -> Result<(), (UpdateStateId, UpdateError)> {
    let deploy =
        Deploy::new(DeployMode::ReadWrite).map_err(|error| (UpdateStateId::Setup, UpdateError::from(error)))?;

    let mut context = Context::new();
    context.put(deploy);
    context.put(
        data.packages
            .iter()
            .map(|path| (*path).to_owned())
            .collect::<Vec<String>>(),
    );
    context.put(TmpPath(data.tmp_path.to_owned()));
    context.put(Subject(data.subject.to_owned()));
    context.put(CommitMessage(data.message.map(str::to_owned)));
    context.put(RequestedBootPlugin(data.boot_plugin.map(str::to_owned)));
    context.put(AllowDowngrade(data.allow_downgrade));
    context.put(AllowConflictFiles(data.allow_conflict_files));
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let orchestrator = assemble();

    let result = run_mutating!(orchestrator, context, data.cancel_token, UpdateStateId, UpdateError);

    data.cancel_token.reset();

    result
}

fn assemble() -> SequentialOrchestrator<UpdateError> {
    SequentialOrchestrator::new(vec![
        Box::new(HookStage {
            trigger: PipelineTrigger::pre(Operation::Update),
        }),
        Box::new(FetchingStage),
        Box::new(PreparationStage),
        Box::new(TransactionStage),
        Box::new(MergeStage),
        Box::new(CheckoutStage),
        Box::new(SwapStage),
        Box::new(HookStage {
            trigger: PipelineTrigger::post(Operation::Update),
        }),
        Box::new(RetentionStage),
    ])
}
