// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::VecDeque;

use composefs::tree::FileSystem;

use upac_types::request::mutated::UninstallRequest;
use uuid::Uuid;

use upac_types::hook::Message;
use upac_types::state::mutated::UninstallStateId;
use upac_types::traits::MessageHook;
use upac_types::{TmpPath, UninstallPackagesTargets};

use upac_macro::ContextValue;

use self::checkout::CheckoutStage;
use self::commit::CommitTransactionStage;
use self::merge::MergeStage;
use self::open::OpenTransactionStage;
use self::preparation::PreparationStage;
use self::remove::RemovePackageStage;
use self::swap::SwapStage;

use crate::composefs::repository::ObjectID;
use crate::database::MemoryDatabase;
use crate::deploy::retention::RetentionStage;
use crate::deploy::{Deploy, DeployMode};
use crate::orchestrator::context::Context;
use crate::orchestrator::{Orchestrator, SequentialOrchestrator, run_mutating, stages};
use crate::plugin::boot::BootPlugin;
use crate::scripts::HookStage;
use crate::scripts::pipeline::{Operation, PipelineTrigger};

pub use self::error::UninstallError;

mod checkout;
mod commit;
mod error;
mod merge;
mod open;
mod preparation;
mod remove;
mod swap;

#[derive(ContextValue)]
pub(crate) struct PackageUuidsToRemove(pub Vec<Uuid>);

pub(crate) struct NewState {
    pub prefix_digest: String,
    pub removed_config_paths: Vec<String>,
}

pub(crate) struct CommitInfo {
    pub subject: String,
    pub message: Option<String>,
}

#[derive(ContextValue)]
pub(crate) struct Purge(pub bool);

#[derive(ContextValue)]
pub(crate) struct RequestedBootPlugin(pub String);
pub(crate) struct ResolvedBootEntry {
    pub plugin: BootPlugin,
    pub entry_name: String,
}

pub(crate) struct RemoveProgress {
    pub pending: VecDeque<Uuid>,
    pub total: u64,
}

pub(crate) struct WorkingState {
    pub tree: FileSystem<ObjectID>,
    pub database: MemoryDatabase,
    pub removed_config_paths: Vec<String>,
}

pub fn run(request: UninstallRequest<'_>) -> Result<(), (UninstallStateId, UninstallError)> {
    let deploy =
        Deploy::new(DeployMode::ReadWrite).map_err(|error| (UninstallStateId::Setup, UninstallError::from(error)))?;

    let targets = UninstallPackagesTargets(request.packages);

    let cancel_token = unsafe { &*request.base.cancel_token };

    let mut context = Context::new();
    context.put(targets);
    context.put(deploy);
    context.put(TmpPath(request.tmp_path.to_owned()));
    context.put(CommitInfo {
        subject: request.subject.to_owned(),
        message: request.message.map(str::to_owned),
    });
    context.put(RequestedBootPlugin(request.boot_plugin.to_owned()));
    context.put(Purge(request.purge));
    context.put(Box::new(Message::new(request.base.on_hook, request.base.hook_ctx)) as Box<dyn MessageHook>);

    let orchestrator = assemble();

    let result = run_mutating!(orchestrator, context, cancel_token, UninstallStateId, UninstallError);

    cancel_token.reset();

    result
}

fn assemble() -> SequentialOrchestrator<UninstallError> {
    SequentialOrchestrator::new(stages![
        HookStage {
            trigger: PipelineTrigger::pre(Operation::Uninstall),
        },
        PreparationStage,
        OpenTransactionStage,
        RemovePackageStage,
        CommitTransactionStage,
        MergeStage,
        CheckoutStage,
        SwapStage,
        HookStage {
            trigger: PipelineTrigger::declarative(Operation::Uninstall),
        },
        HookStage {
            trigger: PipelineTrigger::post(Operation::Uninstall),
        },
        RetentionStage,
    ])
}
