// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::{Path, PathBuf};

use tempfile::TempDir;

use upac_types::decoder::PackageTriggers;
use upac_types::error::ErrorKind;
use upac_types::request::bootstrap::SetupBootstrapImportRequest;
use upac_types::response::bootstrap::SetupBootstrapImportResponse;
use upac_types::state::setup::BootstrapImportStateId;

use upac_decoder_loader::unpack::{PackageTemp, PackageUnpacker};

use upac_deploy::Sysroot;

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::add::AddStage;
use self::commit::CommitStage;
use self::config::ConfigStage;
use self::prepare::PrepareStage;
use self::system::SystemStage;
use self::unpack::UnpackStage;

use super::root_partition;

use crate::MountedTarget;
use crate::layout::mount::DEFAULT_MOUNT_POINT;

mod add;
mod commit;
mod config;
mod prepare;
mod system;
mod unpack;

pub(crate) struct RequestedSource(pub PathBuf);

pub(crate) struct EmptyConfig(pub bool);

pub(crate) struct ScratchDir(pub PathBuf);

pub(crate) struct SourceDir(pub PathBuf);

pub(crate) struct PackageSource {
    pub path: PathBuf,
    pub index: usize,
}

pub(crate) struct UnpackedPackage {
    pub temp: PackageTemp,
    pub triggers: PackageTriggers,
}

pub fn run(
    request: SetupBootstrapImportRequest<'_>,
) -> Result<SetupBootstrapImportResponse, (BootstrapImportStateId, ErrorKind, Option<String>)> {
    let setup_error = |error: ErrorKind| (BootstrapImportStateId::Setup, error, None);

    let deploy_device = root_partition(request.disk, request.deploy_device).map_err(setup_error)?;
    let target = MountedTarget::mount(
        &deploy_device,
        None,
        PathBuf::from(request.mount_point.unwrap_or(DEFAULT_MOUNT_POINT)),
    )
    .map_err(setup_error)?;

    let scratch = TempDir::new_in(request.tmp_path.map_or(target.scratch_dir(), Path::new))
        .map_err(|error| setup_error(error.into()))?;

    let sysroot = Sysroot::init(target.root()).map_err(|error| setup_error(error.into()))?;
    let working = sysroot.empty_prefix().map_err(|error| setup_error(error.into()))?;
    let unpacker = PackageUnpacker::new().map_err(|error| setup_error(error.into()))?;

    let mut context = Context::default();
    context.put(sysroot);
    context.put(working);
    context.put(unpacker);
    context.put(ScratchDir(scratch.path().to_path_buf()));
    context.put(RequestedSource(PathBuf::from(request.source)));
    context.put(EmptyConfig(request.empty_config));

    SequentialOrchestrator::new(stages![
        PrepareStage,
        each::<PackageSource>(UnpackStage, AddStage),
        SystemStage,
        CommitStage,
        ConfigStage,
    ])
    .run_mutating_with_response(&mut context, request.base.cancel_token, &|event| {
        request.base.report_progress(event)
    })
}
