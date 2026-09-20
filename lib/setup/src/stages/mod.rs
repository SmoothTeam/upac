// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::VecDeque;
use std::os::raw::c_void;
use std::path::{Path, PathBuf};

use composefs::tree::FileSystem;

use upac::composefs::repository::ObjectID;
use upac::database::MemoryDatabase;
use upac::errors::CommonError;
use upac::orchestrator::context::Context;
use upac::orchestrator::error::OrchestratorError;
use upac::orchestrator::{Orchestrator, SequentialOrchestrator};
use upac::plugin::decoder::unpack::PackageUnpacker;

use upac_abi::error::ErrorKind;
use upac_abi::hook::CancelToken;
use upac_abi::request::{CSetupExistingRequest, CSetupWholeDiskRequest};
use upac_abi::{FsKind, HookMessageFn, InitramfsGenerator};

use upac_types::decoder::DeclarativeTrigger;
use upac_types::hook::Message;
use upac_types::package::PackageTemp;
use upac_types::request::{BtrfsOptions, GptLayout, PartitionMount, PartitionSpec};
use upac_types::states::SetupStateId;
use upac_types::traits::MessageHook;

use upac_macro::ContextValue;

use self::boot::StageBootStage;
use self::database::EmbedDatabaseStage;
use self::deploy::WriteDeployRecordStage;
use self::enumerate::EnumeratePackagesStage;
use self::import::ImportPackageStage;
use self::kernel::KernelStage;
use self::prepare::PrepareSourceStage;
use self::system::ImportSystemStage;
use self::unpack::UnpackPackageStage;

use super::error::SetupError;
use super::layout::mount::DEFAULT_MOUNT_POINT;
use super::partition::existing_esp_geometry;
use super::target::TargetSysroot;

mod boot;
mod database;
mod deploy;
mod enumerate;
mod import;
mod kernel;
mod prepare;
mod system;
mod unpack;

#[cfg(test)]
#[path = "../../tests/inline/stages.rs"]
mod tests;

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

pub struct SetupExistingData<'data> {
    pub esp_device: &'data str,
    pub deploy_device: &'data str,
    pub deploy_fs: FsKind,
    pub extra_mounts: Vec<PartitionMount>,

    pub mount_point: Option<&'data str>,
    pub source: &'data str,
    pub empty_config: bool,
    pub pinned: bool,

    pub boot_plugin: &'data str,
    pub initramfs_generator: InitramfsGenerator,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'data CancelToken,
}

impl<'data> TryFrom<&'data CSetupExistingRequest> for SetupExistingData<'data> {
    type Error = ErrorKind;

    fn try_from(request: &'data CSetupExistingRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(SetupExistingData {
            esp_device: (&request.esp_device).try_into()?,
            deploy_device: (&request.deploy_device).try_into()?,
            deploy_fs: request.deploy_fs,
            extra_mounts: Vec::try_from(&request.extra_mounts)?,

            mount_point: (&request.mount_point).try_into()?,
            source: (&request.source).try_into()?,
            empty_config: request.empty_config,
            pinned: request.pinned,

            boot_plugin: (&request.boot_plugin).try_into()?,
            initramfs_generator: request.initramfs_generator,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
}

impl SetupExistingData<'_> {
    pub fn mount_point(&self) -> &str {
        self.mount_point.unwrap_or(DEFAULT_MOUNT_POINT)
    }
}

pub struct SetupWholeDiskData<'data> {
    pub device_path: &'data str,
    pub esp_size_mib: u64,
    pub deploy_fs: FsKind,
    pub deploy_size_mib: u64,
    pub extra_partitions: Vec<PartitionSpec>,
    pub force_wipe: bool,

    pub node_size: u32,
    pub sector_size: u32,

    pub mount_point: Option<&'data str>,
    pub source: &'data str,
    pub empty_config: bool,
    pub pinned: bool,

    pub boot_plugin: &'data str,
    pub initramfs_generator: InitramfsGenerator,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'data CancelToken,
}

impl SetupWholeDiskData<'_> {
    pub fn mount_point(&self) -> &str {
        self.mount_point.unwrap_or(DEFAULT_MOUNT_POINT)
    }
}

impl<'data> TryFrom<&'data CSetupWholeDiskRequest> for SetupWholeDiskData<'data> {
    type Error = ErrorKind;

    fn try_from(request: &'data CSetupWholeDiskRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        let gpt = GptLayout::try_from(&request.gpt)?;
        let btrfs = BtrfsOptions::try_from(&request.btrfs)?;

        Ok(SetupWholeDiskData {
            device_path: (&request.device_path).try_into()?,
            esp_size_mib: gpt.esp_size_mib,
            deploy_fs: gpt.deploy_fs,
            deploy_size_mib: gpt.deploy_size_mib,
            extra_partitions: gpt.extra_partitions,
            force_wipe: gpt.force_wipe,

            node_size: btrfs.node_size,
            sector_size: btrfs.sector_size,

            mount_point: (&request.mount_point).try_into()?,
            source: (&request.source).try_into()?,
            empty_config: request.empty_config,
            pinned: request.pinned,

            boot_plugin: (&request.boot_plugin).try_into()?,
            initramfs_generator: request.initramfs_generator,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
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

pub fn run_existing(data: SetupExistingData) -> Result<(), (SetupStateId, SetupError)> {
    let esp_device = Path::new(data.esp_device);
    let (esp_partition_number, esp_starting_lba, esp_ending_lba, esp_unique_partition_guid) =
        existing_esp_geometry(esp_device).map_err(|error| (SetupStateId::Setup, error))?;

    let target = TargetSysroot::new(
        Path::new(data.deploy_device),
        data.deploy_fs,
        esp_device,
        PathBuf::from(data.mount_point()),
        &data.extra_mounts,
        esp_partition_number,
        esp_starting_lba,
        esp_ending_lba,
        esp_unique_partition_guid,
    )
    .map_err(|error| (SetupStateId::Setup, error))?;

    let mut context = Context::new();
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);
    context.put(target);
    context.put(RequestedSource(data.source.to_owned()));
    context.put(EmptyConfig(data.empty_config));
    context.put(Pinned(data.pinned));
    context.put(RequestedBootPlugin(data.boot_plugin.to_owned()));
    context.put(RequestedInitramfsGenerator(data.initramfs_generator));

    let orchestrator = SequentialOrchestrator::new(vec![
        Box::new(PrepareSourceStage),
        Box::new(EnumeratePackagesStage),
        Box::new(UnpackPackageStage),
        Box::new(ImportPackageStage),
        Box::new(ImportSystemStage),
        Box::new(KernelStage),
        Box::new(EmbedDatabaseStage),
        Box::new(WriteDeployRecordStage),
        Box::new(StageBootStage),
    ]);

    let result = if orchestrator.validate(&context).is_err() {
        Err((SetupStateId::Setup, SetupError::from(CommonError::PipelineInvalid)))
    } else {
        orchestrator
            .run_exclusive(&mut context, data.cancel_token)
            .map_err(|failure| match failure {
                OrchestratorError::Setup(lock_error) => (SetupStateId::Setup, SetupError::from(lock_error)),
                OrchestratorError::Stage(index, error) => (SetupStateId::from_stage_index(index), error),
            })
    };

    data.cancel_token.reset();

    result
}

pub fn run_whole_disk(data: SetupWholeDiskData) -> Result<(), (SetupStateId, SetupError)> {
    let target = TargetSysroot::create_whole_disk(&data).map_err(|error| (SetupStateId::Setup, error))?;

    let mut context = Context::new();
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);
    context.put(target);
    context.put(RequestedSource(data.source.to_owned()));
    context.put(EmptyConfig(data.empty_config));
    context.put(Pinned(data.pinned));
    context.put(RequestedBootPlugin(data.boot_plugin.to_owned()));
    context.put(RequestedInitramfsGenerator(data.initramfs_generator));

    let orchestrator = SequentialOrchestrator::new(vec![
        Box::new(PrepareSourceStage),
        Box::new(EnumeratePackagesStage),
        Box::new(UnpackPackageStage),
        Box::new(ImportPackageStage),
        Box::new(ImportSystemStage),
        Box::new(KernelStage),
        Box::new(EmbedDatabaseStage),
        Box::new(WriteDeployRecordStage),
        Box::new(StageBootStage),
    ]);

    let result = if orchestrator.validate(&context).is_err() {
        Err((SetupStateId::Setup, SetupError::from(CommonError::PipelineInvalid)))
    } else {
        orchestrator
            .run_exclusive(&mut context, data.cancel_token)
            .map_err(|failure| match failure {
                OrchestratorError::Setup(lock_error) => (SetupStateId::Setup, SetupError::from(lock_error)),
                OrchestratorError::Stage(index, error) => (SetupStateId::from_stage_index(index), error),
            })
    };

    data.cancel_token.reset();

    result
}
