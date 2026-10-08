// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::decoder::TriggerPosition;
use upac_types::error::ErrorKind;
use upac_types::request::mutated::InstallRequest;
use upac_types::settings::RuntimeSettings;
use upac_types::state::mutated::InstallStateId;
use upac_types::transaction::TransactionKind;

use upac_decoder_loader::unpack::PackageUnpacker;

use upac_deploy::{Sysroot, SysrootMode};

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::import::ImportStage;

use super::TmpPath;
use super::stages::checkout::CheckoutStage;
use super::stages::commit::CommitStage;
use super::stages::deploy::DeployStage;
use super::stages::hooks::HooksStage;
use super::stages::merge::MergeStage;
use super::stages::open::OpenStage;
use super::stages::retention::RetentionStage;
use super::stages::swap::SwapStage;
use super::stages::unpack::UnpackStage;
use super::stages::{CommitInfo, PackageSource, RequestedBootPlugin, UnpackedPackage};

mod import;

pub fn run(request: InstallRequest<'_>) -> Result<(), (InstallStateId, ErrorKind, Option<String>)> {
    if request.packages.is_empty() {
        return Err((InstallStateId::Setup, ErrorKind::InvalidEntry, None));
    }

    let sysroot = Sysroot::new(SysrootMode::ReadWrite).map_err(|error| (InstallStateId::Setup, error.into(), None))?;
    let unpacker = PackageUnpacker::new().map_err(|error| (InstallStateId::Setup, error.into(), None))?;

    let sources: Vec<PackageSource> = request
        .packages
        .iter()
        .enumerate()
        .map(|(index, path)| PackageSource {
            path: (*path).to_owned(),
            index,
        })
        .collect();

    let mut context = Context::default();
    context.put(sysroot);
    context.put(unpacker);
    context.put(sources);
    context.put(TmpPath(request.tmp_path.to_owned()));
    context.put(CommitInfo {
        subject: request.subject.to_owned(),
        message: request.message.map(str::to_owned),
        allow_conflict_files: request.allow_conflict_files,
    });
    context.put(RequestedBootPlugin(request.boot_plugin.to_owned()));

    SequentialOrchestrator::new(stages![
        each::<PackageSource>(UnpackStage::default()),
        HooksStage::new(TriggerPosition::PreInstall),
        OpenStage,
        each::<UnpackedPackage>(ImportStage),
        CommitStage {
            kind: TransactionKind::Install,
        },
        MergeStage,
        DeployStage,
        HooksStage::new(TriggerPosition::PostInstall),
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
