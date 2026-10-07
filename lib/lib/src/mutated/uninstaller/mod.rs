// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use uuid::Uuid;

use upac_types::decoder::TriggerPosition;
use upac_types::error::ErrorKind;
use upac_types::package::PackageInfo;
use upac_types::request::mutated::UninstallRequest;
use upac_types::settings::RuntimeSettings;
use upac_types::state::mutated::UninstallStateId;
use upac_types::transaction::TransactionKind;

use upac_deploy::{Sysroot, SysrootMode};

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::prepare::PrepareStage;
use self::remove::RemoveStage;

use super::stages::checkout::CheckoutStage;
use super::stages::commit::CommitStage;
use super::stages::deploy::DeployStage;
use super::stages::hooks::HooksStage;
use super::stages::merge::MergeStage;
use super::stages::open::OpenStage;
use super::stages::retention::RetentionStage;
use super::stages::swap::SwapStage;
use super::stages::{CommitInfo, RequestedBootPlugin};

mod prepare;
mod remove;

pub(crate) struct RequestedPackages(pub Vec<PackageInfo>);

pub(crate) struct RemovalTarget(pub Uuid);

pub(crate) struct Purge(pub bool);

pub fn run(request: UninstallRequest<'_>) -> Result<(), (UninstallStateId, ErrorKind)> {
    if request.packages.is_empty() {
        return Err((UninstallStateId::Setup, ErrorKind::InvalidEntry));
    }

    let sysroot = Sysroot::new(SysrootMode::ReadWrite).map_err(|error| (UninstallStateId::Setup, error.into()))?;

    let mut context = Context::default();
    context.put(sysroot);
    context.put(RequestedPackages(request.packages));
    context.put(Purge(request.purge));
    context.put(CommitInfo {
        subject: request.subject.to_owned(),
        message: request.message.map(str::to_owned),
        allow_conflict_files: false,
    });
    context.put(RequestedBootPlugin(request.boot_plugin.to_owned()));

    SequentialOrchestrator::new(stages![
        OpenStage,
        PrepareStage,
        HooksStage::new(TriggerPosition::PreRemove),
        each::<RemovalTarget>(RemoveStage),
        CommitStage {
            kind: TransactionKind::Uninstall,
        },
        MergeStage,
        DeployStage,
        HooksStage::new(TriggerPosition::PostRemove),
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
