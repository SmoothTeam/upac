// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::{Path, PathBuf};

use upac_types::error::ErrorKind;
use upac_types::request::bootstrap::SetupBootstrapDeployRequest;
use upac_types::request::partition::PartitionKind;
use upac_types::state::setup::BootstrapDeployStateId;

use upac_composefs::Digest;
use upac_composefs::fs::WrittenFile;

use upac_deploy::Sysroot;

use upac_orchestrator::context::Context;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

use self::boot::BootStage;
use self::register::RegisterStage;

use crate::MountedTarget;
use crate::gpt::{Disk, Partition};
use crate::layout::mount::DEFAULT_MOUNT_POINT;

mod boot;
mod register;

pub(crate) struct RequestedDeploy {
    pub prefix_digest: Digest,
    pub config_digest: Digest,
    pub pinned: bool,
}

pub(crate) struct RequestedBootPlugin(pub String);

pub(crate) struct EspMountPoint(pub PathBuf);

pub(crate) struct DeployedPrefix {
    pub digest: Digest,
    pub next_written: WrittenFile,
}

pub fn run(
    request: SetupBootstrapDeployRequest<'_>,
) -> Result<(), (BootstrapDeployStateId, ErrorKind, Option<String>)> {
    let setup_error = |error: ErrorKind| (BootstrapDeployStateId::Setup, error, None);

    let requested = RequestedDeploy {
        prefix_digest: Digest::from_hex(request.prefix_digest).map_err(|_| setup_error(ErrorKind::InvalidEntry))?,
        config_digest: Digest::from_hex(request.config_digest).map_err(|_| setup_error(ErrorKind::InvalidEntry))?,
        pinned: request.pinned,
    };

    let (esp, deploy_device) =
        target_partitions(request.disk, request.esp_device, request.deploy_device).map_err(setup_error)?;
    if !esp.is_esp() {
        return Err(setup_error(ErrorKind::WrongPartitionType));
    }

    let target = MountedTarget::mount(
        &deploy_device,
        Some(esp.path()),
        PathBuf::from(request.mount_point.unwrap_or(DEFAULT_MOUNT_POINT)),
    )
    .map_err(setup_error)?;
    let esp_mount_point = target
        .esp_mount_point()
        .ok_or_else(|| setup_error(ErrorKind::NotFound))?
        .to_path_buf();

    let sysroot = Sysroot::open(target.root()).map_err(|error| setup_error(error.into()))?;

    let mut context = Context::default();
    context.put(sysroot);
    context.put(requested);
    context.put(esp);
    context.put(EspMountPoint(esp_mount_point));
    context.put(RequestedBootPlugin(request.boot_plugin.to_owned()));

    SequentialOrchestrator::new(stages![RegisterStage, BootStage]).run_mutating(
        &mut context,
        request.base.cancel_token,
        &|event| request.base.report_progress(event),
    )
}

fn target_partitions(
    disk: Option<&str>, esp_device: Option<&str>, deploy_device: Option<&str>,
) -> Result<(Partition, PathBuf), ErrorKind> {
    match (disk, esp_device, deploy_device) {
        (Some(disk), None, None) => {
            let disk = Disk::open(Path::new(disk))?;
            let deploy = disk.find(PartitionKind::Root)?;

            Ok((disk.find(PartitionKind::Esp)?, deploy.path().to_path_buf()))
        }
        (None, Some(esp_device), Some(deploy_device)) => {
            Ok((Partition::open(Path::new(esp_device))?, PathBuf::from(deploy_device)))
        }
        _ => Err(ErrorKind::InvalidEntry),
    }
}
