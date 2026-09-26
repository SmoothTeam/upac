// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::VecDeque;
use std::path::PathBuf;

use composefs::tree::FileSystem;

use uuid::Uuid;

use upac_abi::response::entry::{DiffFileSource, FileDiffKind};

use upac_types::TmpPath;
use upac_types::package::PackageInfo;
use upac_types::request::mutated::FilesRequest;
use upac_types::state::mutated::FilesStateId;

use upac_macro::ContextValue;

use upac_boot_loader::BootPlugin;

use upac_composefs::repository::ObjectID;

use upac_database::MemoryDatabase;

use upac_deploy::retention::RetentionStage;
use upac_deploy::{Deploy, DeployMode};

use upac_hooks::HookStage;
use upac_hooks::pipeline::{Operation, PipelineTrigger};

use upac_orchestrator::context::Context;
use upac_orchestrator::{Orchestrator, SequentialOrchestrator, run_mutating, stages};

use self::apply::ApplyFileStage;
use self::checkout::CheckoutStage;
use self::commit::CommitTransactionStage;
use self::open::OpenTransactionStage;
use self::swap::SwapStage;

pub use self::error::FilesError;

mod apply;
mod checkout;
mod commit;
mod error;
mod open;
mod swap;

pub(crate) struct RequestedFileOperation {
    pub kind: FileDiffKind,
    pub scope: DiffFileSource,
}
pub(crate) struct RequestedFilePackage {
    pub name: String,
    pub arch: String,
    pub arch_sub: Option<String>,
}

#[derive(ContextValue)]
pub(crate) struct NewPrefixDigest(pub String);

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

pub(crate) struct FileProgress {
    pub pending: VecDeque<String>,
    pub total: u64,
}

pub(crate) struct WorkingState {
    pub tree: FileSystem<ObjectID>,
    pub database: MemoryDatabase,
}

pub(crate) struct ApplyTarget {
    pub uuid: Uuid,
    pub config_upper_dir: PathBuf,
}

pub fn run(request: FilesRequest<'_>) -> Result<(), (FilesStateId, FilesError)> {
    let deploy = Deploy::new(DeployMode::ReadWrite).map_err(|error| (FilesStateId::Setup, FilesError::from(error)))?;
    let cancel_token = unsafe { &*request.base.cancel_token };

    let file_package_c =
        unsafe { request.file_package.as_ref() }.ok_or((FilesStateId::Setup, FilesError::PackageNotFound))?;
    let file_package =
        PackageInfo::try_from(file_package_c).map_err(|_| (FilesStateId::Setup, FilesError::PackageNotFound))?;

    let mut context = Context::new();
    context.put(deploy);
    context.put(
        request
            .files
            .iter()
            .map(|path| (*path).to_owned())
            .collect::<Vec<String>>(),
    );
    context.put(RequestedFileOperation {
        kind: request.file_kind,
        scope: request.scope,
    });
    context.put(RequestedFilePackage {
        name: file_package.name,
        arch: file_package.arch,
        arch_sub: file_package.arch_sub,
    });
    context.put(TmpPath(request.tmp_path.to_owned()));
    context.put(CommitInfo {
        subject: request.subject.to_owned(),
        message: request.message.map(str::to_owned),
    });
    context.put(RequestedBootPlugin(request.boot_plugin.to_owned()));
    context.put(request.base.message_hook());

    let orchestrator = assemble();

    let result = run_mutating!(orchestrator, context, cancel_token, FilesStateId, FilesError);

    cancel_token.reset();

    result
}

fn assemble() -> SequentialOrchestrator<FilesError> {
    SequentialOrchestrator::new(stages![
        HookStage {
            trigger: PipelineTrigger::pre(Operation::Files),
        },
        OpenTransactionStage,
        ApplyFileStage,
        CommitTransactionStage,
        CheckoutStage,
        SwapStage,
        HookStage {
            trigger: PipelineTrigger::post(Operation::Files),
        },
        RetentionStage,
    ])
}
