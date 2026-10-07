// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::error::ErrorKind;
use upac_types::request::mutated::DetachRequest;
use upac_types::settings::RuntimeSettings;
use upac_types::state::mutated::DetachStateId;
use upac_types::transaction::TransactionKind;

use upac_deploy::{Sysroot, SysrootMode};

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::drop::DropStage;

use super::stages::checkout::CheckoutStage;
use super::stages::commit::CommitStage;
use super::stages::deploy::DeployStage;
use super::stages::merge::MergeStage;
use super::stages::open::OpenStage;
use super::stages::resolve::ResolveStage;
use super::stages::retention::RetentionStage;
use super::stages::swap::SwapStage;
use super::stages::{CommitInfo, DetachItem, RequestedBootPlugin, RequestedPackage, RequestedScope};

mod drop;

pub fn run(request: DetachRequest<'_>) -> Result<(), (DetachStateId, ErrorKind)> {
    if request.files.is_empty() {
        return Err((DetachStateId::Setup, ErrorKind::InvalidEntry));
    }

    let items: Vec<DetachItem> = request
        .files
        .iter()
        .map(|target| DetachItem((*target).to_owned()))
        .collect();
    let sysroot = Sysroot::new(SysrootMode::ReadWrite).map_err(|error| (DetachStateId::Setup, error.into()))?;

    let mut context = Context::default();
    context.put(sysroot);
    context.put(items);
    context.put(RequestedScope(request.scope));
    context.put(RequestedPackage(request.file_package));
    context.put(CommitInfo {
        subject: request.subject.to_owned(),
        message: request.message.map(str::to_owned),
        allow_conflict_files: false,
    });
    context.put(RequestedBootPlugin(request.boot_plugin.to_owned()));

    SequentialOrchestrator::new(stages![
        OpenStage,
        ResolveStage,
        each::<DetachItem>(DropStage),
        CommitStage {
            kind: TransactionKind::Files,
        },
        MergeStage,
        DeployStage,
        CheckoutStage,
        SwapStage,
        RetentionStage {
            retention_depth: RuntimeSettings::load().gc.retention_depth,
        },
    ])
    .run_mutating(&mut context, request.base.cancel_token, &|event| {
        request.base.report_progress(event)
    })
}
