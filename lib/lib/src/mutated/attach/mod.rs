// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::error::ErrorKind;
use upac_types::request::mutated::AttachRequest;
use upac_types::settings::RuntimeSettings;
use upac_types::state::mutated::AttachStateId;

use upac_deploy::{Sysroot, SysrootMode};

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::commit::CommitStage;
use self::put::PutStage;

use super::TmpPath;
use super::stages::checkout::CheckoutStage;
use super::stages::deploy::DeployStage;
use super::stages::merge::MergeStage;
use super::stages::open::OpenStage;
use super::stages::resolve::ResolveStage;
use super::stages::retention::RetentionStage;
use super::stages::swap::SwapStage;
use super::stages::{AttachItem, CommitInfo, RequestedBootPlugin, RequestedPackage, ScopedPath};

use crate::report_progress;

mod commit;
mod put;

pub fn run(request: AttachRequest<'_>) -> Result<(), (AttachStateId, ErrorKind)> {
    if request.files.is_empty() {
        return Err((AttachStateId::Setup, ErrorKind::InvalidEntry));
    }

    let items = request
        .files
        .iter()
        .map(|transfer| {
            Ok(AttachItem {
                source: transfer.source.into(),
                target: ScopedPath::new(request.scope, transfer.target)?,
            })
        })
        .collect::<Result<Vec<_>, ErrorKind>>()
        .map_err(|error| (AttachStateId::Setup, error))?;
    let sysroot = Sysroot::new(SysrootMode::ReadWrite).map_err(|error| (AttachStateId::Setup, error.into()))?;

    let mut context = Context::default();
    context.put(sysroot);
    context.put(items);
    context.put(RequestedPackage(request.file_package));
    context.put(TmpPath(request.tmp_path.to_owned()));
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
        CommitStage,
        MergeStage,
        DeployStage,
        CheckoutStage,
        SwapStage,
        RetentionStage {
            retention_depth: RuntimeSettings::load().gc.retention_depth,
        },
    ])
    .run_mutating(&mut context, request.base.cancel_token, &|event| {
        report_progress(&request.base, event)
    })
}
