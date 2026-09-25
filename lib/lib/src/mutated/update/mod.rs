// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::VecDeque;

use composefs::tree::FileSystem;

use upac_types::TmpPath;
use upac_types::decoder::DeclarativeTrigger;
use upac_types::package::PackageTemp;
use upac_types::request::mutated::UpdateRequest;
use upac_types::state::mutated::UpdateStateId;

use upac_macro::ContextValue;

use upac_boot_loader::BootPlugin;

use upac_composefs::repository::ObjectID;

use upac_database::MemoryDatabase;

use upac_decoder_loader::unpack::PackageUnpacker;

use upac_deploy::retention::RetentionStage;
use upac_deploy::{Deploy, DeployMode};

use upac_hooks::HookStage;
use upac_hooks::pipeline::{Operation, PipelineTrigger};

use upac_orchestrator::context::Context;
use upac_orchestrator::{Orchestrator, SequentialOrchestrator, run_mutating, stages};

use self::checkout::CheckoutStage;
use self::commit::CommitTransactionStage;
use self::fetching::FetchingStage;
use self::import::ImportPackageStage;
use self::merge::MergeStage;
use self::open::OpenTransactionStage;
use self::preparation::PreparationStage;
use self::swap::SwapStage;

use crate::errors::CommonError;

pub use self::error::UpdateError;

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
    pub removed_config_paths: Vec<String>,
}

pub(crate) struct CommitInfo {
    pub subject: String,
    pub message: Option<String>,
}

#[derive(ContextValue)]
pub(crate) struct RequestedBootPlugin(pub String);
pub(crate) struct ResolvedBootEntry {
    pub plugin: BootPlugin,
    pub entry_name: String,
}

#[derive(ContextValue)]
pub(crate) struct AllowDowngrade(pub bool);
#[derive(ContextValue)]
pub(crate) struct AllowConflictFiles(pub bool);

pub(crate) struct UnpackState {
    pub pending_paths: VecDeque<String>,
    pub unpacker: PackageUnpacker,
}

pub(crate) struct ImportProgress {
    pub pending: VecDeque<(PackageTemp, DeclarativeTrigger)>,
    pub total: u64,
}

pub(crate) struct ImportedState {
    pub tree: FileSystem<ObjectID>,
    pub config_defaults: FileSystem<ObjectID>,
    pub database: MemoryDatabase,
    pub removed_config_paths: Vec<String>,
}

pub fn run(request: UpdateRequest<'_>) -> Result<(), (UpdateStateId, UpdateError)> {
    let deploy =
        Deploy::new(DeployMode::ReadWrite).map_err(|error| (UpdateStateId::Setup, UpdateError::from(error)))?;
    let unpacker = PackageUnpacker::new()
        .map_err(|error| (UpdateStateId::Setup, UpdateError::from(CommonError::Decoder(error))))?;

    let total_packages = request.packages.len() as u64;
    let cancel_token = unsafe { &*request.base.cancel_token };

    let mut context = Context::new();
    context.put(deploy);
    context.put(UnpackState {
        pending_paths: request.packages.iter().map(|path| (*path).to_owned()).collect(),
        unpacker,
    });
    context.put(ImportProgress {
        pending: VecDeque::new(),
        total: total_packages,
    });
    context.put(TmpPath(request.tmp_path.to_owned()));
    context.put(CommitInfo {
        subject: request.subject.to_owned(),
        message: request.message.map(str::to_owned),
    });
    context.put(RequestedBootPlugin(request.boot_plugin.to_owned()));
    context.put(AllowDowngrade(request.allow_downgrade));
    context.put(AllowConflictFiles(request.allow_conflict_files));
    context.put(request.base.message_hook());

    let orchestrator = assemble();

    let result = run_mutating!(orchestrator, context, cancel_token, UpdateStateId, UpdateError);

    cancel_token.reset();

    result
}

fn assemble() -> SequentialOrchestrator<UpdateError> {
    SequentialOrchestrator::new(stages![
        HookStage {
            trigger: PipelineTrigger::pre(Operation::Update),
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
            trigger: PipelineTrigger::declarative(Operation::Update),
        },
        HookStage {
            trigger: PipelineTrigger::post(Operation::Update),
        },
        RetentionStage,
    ])
}
