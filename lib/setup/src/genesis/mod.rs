// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::path::{Path, PathBuf};

use upac::composefs::repository::ObjectID;
use upac::errors::CommonError;
use upac::orchestrator::error::OrchestratorError;
use upac::orchestrator::{Context, Orchestrator, SequentialOrchestrator};

use upac_abi::hook::{Message, MessageHook};

use self::deploy::WriteDeployRecordStage;
use self::entry::StageBootStage;
use self::meta::ReadMetaStage;
use self::trees::ImportTreesStage;

use crate::data::{SetupExistingData, SetupWholeDiskData};
use crate::error::SetupError;
use crate::target::TargetSysroot;

mod deploy;
mod entry;
mod meta;
mod trees;

struct GenesisInput {
    source_dir: String,
    meta_filename: Option<String>,
    empty_config: bool,
    pinned: bool,
    boot_plugin: Option<String>,
}

impl From<&SetupExistingData<'_>> for GenesisInput {
    fn from(data: &SetupExistingData<'_>) -> Self {
        GenesisInput {
            source_dir: data.source_dir.to_owned(),
            meta_filename: data.meta_filename.map(str::to_owned),
            empty_config: data.empty_config,
            pinned: data.pinned,
            boot_plugin: data.boot_plugin.map(str::to_owned),
        }
    }
}

impl From<&SetupWholeDiskData<'_>> for GenesisInput {
    fn from(data: &SetupWholeDiskData<'_>) -> Self {
        GenesisInput {
            source_dir: data.source_dir.to_owned(),
            meta_filename: data.meta_filename.map(str::to_owned),
            empty_config: data.empty_config,
            pinned: data.pinned,
            boot_plugin: data.boot_plugin.map(str::to_owned),
        }
    }
}

struct PrefixDigest(ObjectID);

struct ConfigDigest(ObjectID);

impl SetupExistingData<'_> {
    pub fn run(&self) -> Result<(), SetupError> {
        let target = TargetSysroot::new(
            Path::new(self.deploy_device),
            self.deploy_fs,
            Path::new(self.esp_device),
            PathBuf::from(self.mount_point()),
            &self.extra_mounts,
        )?;

        let mut context = Context::new();
        context.put(Box::new(Message::new(self.hook_message, self.hook_message_context)) as Box<dyn MessageHook>);
        context.put(target);
        context.put(GenesisInput::from(self));

        let orchestrator = SequentialOrchestrator::new(vec![
            Box::new(ReadMetaStage),
            Box::new(ImportTreesStage),
            Box::new(WriteDeployRecordStage),
            Box::new(StageBootStage),
        ]);

        let result = if orchestrator.validate(&context).is_err() {
            Err(SetupError::from(CommonError::PipelineInvalid))
        } else {
            orchestrator
                .run_exclusive(&mut context, self.cancel_token)
                .map_err(|failure| match failure {
                    OrchestratorError::Setup(lock_error) => SetupError::from(lock_error),
                    OrchestratorError::Stage(_index, error) => error,
                })
        };

        self.cancel_token.reset();

        result
    }
}

impl SetupWholeDiskData<'_> {
    pub fn run(&self) -> Result<(), SetupError> {
        let target = TargetSysroot::create_whole_disk(self)?;

        let mut context = Context::new();
        context.put(Box::new(Message::new(self.hook_message, self.hook_message_context)) as Box<dyn MessageHook>);
        context.put(target);
        context.put(GenesisInput::from(self));

        let orchestrator = SequentialOrchestrator::new(vec![
            Box::new(ReadMetaStage),
            Box::new(ImportTreesStage),
            Box::new(WriteDeployRecordStage),
            Box::new(StageBootStage),
        ]);

        let result = if orchestrator.validate(&context).is_err() {
            Err(SetupError::from(CommonError::PipelineInvalid))
        } else {
            orchestrator
                .run_exclusive(&mut context, self.cancel_token)
                .map_err(|failure| match failure {
                    OrchestratorError::Setup(lock_error) => SetupError::from(lock_error),
                    OrchestratorError::Stage(_index, error) => error,
                })
        };

        self.cancel_token.reset();

        result
    }
}
