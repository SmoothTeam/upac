// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::{Path, PathBuf};

use tempfile::TempDir;

use upac_types::error::ErrorKind;
use upac_types::request::bootstrap::{InitramfsGenerator, SetupBootstrapKernelRequest};
use upac_types::response::bootstrap::SetupBootstrapKernelResponse;
use upac_types::state::setup::BootstrapKernelStateId;

use upac_composefs::Digest;

use upac_deploy::Sysroot;

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::commit::CommitStage;
use self::export::ExportStage;
use self::generate::GenerateStage;

use super::root_partition;

use crate::MountedTarget;
use crate::layout::mount::DEFAULT_MOUNT_POINT;

mod commit;
mod export;
mod generate;

pub(crate) struct ExportDir(pub PathBuf);

pub(crate) struct RequestedBootPlugin(pub String);

pub(crate) struct RequestedInitramfsGenerator(pub InitramfsGenerator);

pub(crate) struct KernelVersion(pub String);

pub(crate) struct KernelImage(pub PathBuf);

pub fn run(
    request: SetupBootstrapKernelRequest<'_>,
) -> Result<SetupBootstrapKernelResponse, (BootstrapKernelStateId, ErrorKind)> {
    let setup_error = |error: ErrorKind| (BootstrapKernelStateId::Setup, error);

    let prefix_digest = Digest::from_hex(request.prefix_digest).map_err(|_| setup_error(ErrorKind::InvalidEntry))?;

    let deploy_device = root_partition(request.disk, request.deploy_device).map_err(setup_error)?;
    let target = MountedTarget::mount(
        &deploy_device,
        None,
        PathBuf::from(request.mount_point.unwrap_or(DEFAULT_MOUNT_POINT)),
    )
    .map_err(setup_error)?;

    let scratch = TempDir::new_in(request.tmp_path.map_or(target.scratch_dir(), Path::new))
        .map_err(|error| setup_error(error.into()))?;

    let sysroot = Sysroot::open(target.root()).map_err(|error| setup_error(error.into()))?;
    let prefix_tree = sysroot
        .repo()
        .open_tree(&prefix_digest)
        .map_err(|error| setup_error(error.into()))?;

    let mut context = Context::default();
    context.put(prefix_tree);
    context.put(ExportDir(scratch.path().to_path_buf()));
    context.put(RequestedBootPlugin(request.boot_plugin.to_owned()));
    context.put(RequestedInitramfsGenerator(request.initramfs_generator));

    SequentialOrchestrator::new(stages![ExportStage, GenerateStage, CommitStage]).run_mutating_with_response(
        &mut context,
        request.base.cancel_token,
        &|event| request.base.report_progress(event),
    )
}
