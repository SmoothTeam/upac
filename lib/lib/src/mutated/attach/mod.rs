// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::error::ErrorKind;
use upac_types::request::mutated::AttachRequest;
use upac_types::settings::RuntimeSettings;
use upac_types::state::mutated::AttachStateId;
use upac_types::transaction::TransactionKind;

use upac_deploy::{Sysroot, SysrootMode};

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::put::PutStage;

use super::stages::checkout::CheckoutStage;
use super::stages::commit::CommitStage;
use super::stages::deploy::DeployStage;
use super::stages::merge::MergeStage;
use super::stages::open::OpenStage;
use super::stages::resolve::ResolveStage;
use super::stages::retention::RetentionStage;
use super::stages::swap::SwapStage;
use super::stages::{AttachItem, CommitInfo, RequestedBootPlugin, RequestedPackage, RequestedScope};

mod put;

pub fn run(request: AttachRequest<'_>) -> Result<(), (AttachStateId, ErrorKind)> {
    if request.files.is_empty() {
        return Err((AttachStateId::Setup, ErrorKind::InvalidEntry));
    }

    let items: Vec<AttachItem> = request
        .files
        .iter()
        .map(|transfer| AttachItem {
            source: transfer.source.into(),
            target: transfer.target.to_owned(),
        })
        .collect();
    let sysroot = Sysroot::new(SysrootMode::ReadWrite).map_err(|error| (AttachStateId::Setup, error.into()))?;

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
        each::<AttachItem>(PutStage),
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
