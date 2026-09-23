// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::VecDeque;

use composefs::tree::FileSystem;

use upac_types::TmpPath;
use upac_types::decoder::DeclarativeTrigger;
use upac_types::hook::Message;
use upac_types::package::PackageTemp;
use upac_types::request::mutated::InstallRequest;
use upac_types::state::mutated::InstallStateId;
use upac_types::traits::MessageHook;

use upac_macro::ContextValue;

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
use crate::orchestrator::context::Context;
use crate::orchestrator::{Orchestrator, SequentialOrchestrator, run_mutating, stages};
use crate::plugin::boot::BootPlugin;
use crate::plugin::decoder::unpack::PackageUnpacker;
use crate::scripts::HookStage;
use crate::scripts::pipeline::{Operation, PipelineTrigger};

pub use self::error::InstallError;

mod checkout;
mod commit;
mod error;
mod fetching;
mod import;
mod merge;
mod open;
mod preparation;
mod swap;

pub(crate) struct NewState {
    pub prefix_digest: String,
    pub config_defaults: FileSystem<ObjectID>,
}

pub(crate) struct CommitInfo {
    pub subject: String,
    pub message: Option<String>,
    pub allow_conflict_files: bool,
}

#[derive(ContextValue)]
pub(crate) struct RequestedBootPlugin(pub String);
pub(crate) struct ResolvedBootEntry {
    pub plugin: BootPlugin,
    pub entry_name: String,
}

pub(crate) struct UnpackState {
    pub pending_paths: VecDeque<String>,
    pub unpacker: PackageUnpacker,
}

pub(crate) struct InstallProgress {
    pub pending: VecDeque<(PackageTemp, DeclarativeTrigger)>,
    pub total: u64,
}

pub(crate) struct ImportedState {
    pub tree: FileSystem<ObjectID>,
    pub config_defaults: FileSystem<ObjectID>,
    pub database: MemoryDatabase,
}

pub fn run(request: InstallRequest<'_>) -> Result<(), (InstallStateId, InstallError)> {
    let deploy =
        Deploy::new(DeployMode::ReadWrite).map_err(|error| (InstallStateId::Setup, InstallError::from(error)))?;
    let unpacker = PackageUnpacker::new()
        .map_err(|error| (InstallStateId::Setup, InstallError::from(CommonError::Decoder(error))))?;

    let total_packages = request.packages.len() as u64;
    let cancel_token = unsafe { &*request.base.cancel_token };

    let mut context = Context::new();
    context.put(deploy);
    context.put(UnpackState {
        pending_paths: request.packages.iter().map(|path| (*path).to_owned()).collect(),
        unpacker,
    });
    context.put(InstallProgress {
        pending: VecDeque::new(),
        total: total_packages,
    });
    context.put(TmpPath(request.tmp_path.to_owned()));
    context.put(CommitInfo {
        subject: request.subject.to_owned(),
        message: request.message.map(str::to_owned),
        allow_conflict_files: request.allow_conflict_files,
    });
    context.put(RequestedBootPlugin(request.boot_plugin.to_owned()));
    context.put(Box::new(Message::new(request.base.on_hook, request.base.hook_ctx)) as Box<dyn MessageHook>);

    let orchestrator = assemble();

    let result = run_mutating!(orchestrator, context, cancel_token, InstallStateId, InstallError);

    cancel_token.reset();

    result
}

fn assemble() -> SequentialOrchestrator<InstallError> {
    SequentialOrchestrator::new(stages![
        HookStage {
            trigger: PipelineTrigger::pre(Operation::Install),
        },
        FetchingStage,
        PreparationStage,
        OpenTransactionStage,
        ImportPackageStage,
        CommitTransactionStage,
        MergeStage,
        CheckoutStage,
        SwapStage,
        HookStage {
            trigger: PipelineTrigger::declarative(Operation::Install),
        },
        HookStage {
            trigger: PipelineTrigger::post(Operation::Install),
        },
        RetentionStage,
    ])
}
