// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::VecDeque;
use std::path::PathBuf;

use composefs::tree::FileSystem;

use upac::composefs::repository::ObjectID;
use upac::database::MemoryDatabase;
use upac::errors::CommonError;
use upac::orchestrator::context::Context;
use upac::orchestrator::{Orchestrator, SequentialOrchestrator, run_mutating, stages};
use upac::plugin::decoder::unpack::PackageUnpacker;

use upac_abi::{FsKind, InitramfsGenerator};

use upac_types::decoder::DeclarativeTrigger;
use upac_types::package::PackageTemp;
use upac_types::request::bootstrap::SetupBootstrapRequest;
use upac_types::state::setup::BootstrapStateId;

use upac_macro::ContextValue;

use self::boot::StageBootStage;
use self::database::EmbedDatabaseStage;
use self::deploy::WriteDeployRecordStage;
use self::enumerate::EnumeratePackagesStage;
use self::error::BootstrapError;
use self::import::ImportPackageStage;
use self::kernel::KernelStage;
use self::mount::MountStage;
use self::prepare::PrepareSourceStage;
use self::system::ImportSystemStage;
use self::unpack::UnpackPackageStage;

use crate::layout::mount::DEFAULT_MOUNT_POINT;

pub mod error;

mod boot;
mod database;
mod deploy;
mod enumerate;
mod import;
mod kernel;
mod mount;
mod prepare;
mod system;
mod unpack;

macro_rules! import_if_dir {
    ($repository:expr, $tree:expr, $source:expr, $import_ctx:expr, $cancel:expr) => {
        if $source.is_dir() {
            upac::composefs::file::FileHandle::new(::std::path::PathBuf::new()).import_directory(
                $repository,
                $tree,
                $source,
                $import_ctx,
                $cancel,
                &mut |_| {},
            )?
        } else {
            Vec::new()
        }
    };
}
pub(crate) use import_if_dir;

pub(crate) struct RequestedMount {
    pub esp_device: PathBuf,
    pub deploy_device: PathBuf,
    pub deploy_fs: FsKind,
    pub mount_point: PathBuf,
}

#[derive(ContextValue)]
pub(crate) struct RequestedSource(pub String);

#[derive(ContextValue)]
pub(crate) struct EmptyConfig(pub bool);

#[derive(ContextValue)]
pub(crate) struct Pinned(pub bool);

#[derive(ContextValue)]
pub(crate) struct RequestedBootPlugin(pub String);

#[derive(ContextValue)]
pub(crate) struct RequestedInitramfsGenerator(pub InitramfsGenerator);

#[derive(ContextValue)]
pub(crate) struct ResolvedSourceDir(pub PathBuf);

#[derive(ContextValue)]
pub(crate) struct PrefixTree(pub FileSystem<ObjectID>);

pub(crate) struct UnpackState {
    pub pending_paths: VecDeque<String>,
    pub unpacker: PackageUnpacker,
}

pub(crate) struct SetupProgress {
    pub pending: VecDeque<(PackageTemp, DeclarativeTrigger)>,
    pub total: u64,
}

pub(crate) struct ConfigState {
    pub config_tree: FileSystem<ObjectID>,
    pub database: MemoryDatabase,
}

pub(crate) struct DeployDigests {
    pub prefix: ObjectID,
    pub config: ObjectID,
}

pub fn run(request: SetupBootstrapRequest<'_>) -> Result<(), (BootstrapStateId, BootstrapError)> {
    let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or((
        BootstrapStateId::Setup,
        BootstrapError::from(CommonError::PipelineInvalid),
    ))?;

    let mut context = Context::new();
    context.put(request.base.message_hook());
    context.put(RequestedMount {
        esp_device: PathBuf::from(request.esp_device),
        deploy_device: PathBuf::from(request.deploy_device),
        deploy_fs: request.deploy_fs,
        mount_point: PathBuf::from(request.mount_point.unwrap_or(DEFAULT_MOUNT_POINT)),
    });
    context.put(RequestedSource(request.source.to_owned()));
    context.put(EmptyConfig(request.empty_config));
    context.put(Pinned(request.pinned));
    context.put(RequestedBootPlugin(request.boot_plugin.to_owned()));
    context.put(RequestedInitramfsGenerator(request.initramfs_generator));

    let orchestrator = SequentialOrchestrator::new(stages![
        MountStage,
        PrepareSourceStage,
        EnumeratePackagesStage,
        UnpackPackageStage,
        ImportPackageStage,
        ImportSystemStage,
        KernelStage,
        EmbedDatabaseStage,
        WriteDeployRecordStage,
        StageBootStage,
    ]);

    let result = run_mutating!(orchestrator, context, cancel_token, BootstrapStateId, BootstrapError);

    cancel_token.reset();

    result
}
