// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::{Path, PathBuf};

use upac::errors::CommonError;
use upac::orchestrator::error::OrchestratorError;
use upac::orchestrator::{Context, Orchestrator, SequentialOrchestrator};

use upac_abi::hook::{Message, MessageHook};

use upac_macro::{FromStageIndex, StageKey};

use self::database::CreateDatabaseStage;
use self::deploy::WriteDeployRecordStage;
use self::embed::EmbedDatabaseStage;
use self::entry::StageBootStage;
use self::files::InsertFileEntryStage;
use self::meta::ReadMetaStage;
use self::source::PrepareSourceStage;
use self::trees::ImportTreesStage;

use crate::data::{SetupExistingData, SetupWholeDiskData};
use crate::error::SetupError;
use crate::target::TargetSysroot;
use crate::types::GenesisInput;

mod database;
mod deploy;
mod embed;
mod entry;
mod files;
mod meta;
mod source;
mod trees;

macro_rules! ctx_get {
    ($context:expr, $ty:ty) => {
        $context.get::<$ty>().ok_or(upac::errors::CommonError::MissingResult)?
    };
}
pub(crate) use ctx_get;

macro_rules! ctx_take {
    ($context:expr, $ty:ty) => {
        $context.take::<$ty>().ok_or(upac::errors::CommonError::MissingResult)?
    };
}
pub(crate) use ctx_take;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex, StageKey)]
pub enum GenesisStage {
    PrepareSource = 0,
    ReadMeta = 1,
    ImportTrees = 2,
    CreateDatabase = 3,
    InsertFileEntry = 4,
    EmbedDatabase = 5,
    WriteDeployRecord = 6,
    StageBoot = 7,
    Setup = 8,
}

impl SetupExistingData<'_> {
    pub fn run(&self) -> Result<(), (GenesisStage, SetupError)> {
        let target = TargetSysroot::new(
            Path::new(self.deploy_device),
            self.deploy_fs,
            Path::new(self.esp_device),
            PathBuf::from(self.mount_point()),
            &self.extra_mounts,
        )
        .map_err(|error| (GenesisStage::Setup, error))?;

        let mut context = Context::new();
        context.put(Box::new(Message::new(self.hook_message, self.hook_message_context)) as Box<dyn MessageHook>);
        context.put(target);
        context.put(GenesisInput::from(self));

        let orchestrator = SequentialOrchestrator::new(vec![
            Box::new(PrepareSourceStage),
            Box::new(ReadMetaStage),
            Box::new(ImportTreesStage),
            Box::new(CreateDatabaseStage),
            Box::new(InsertFileEntryStage),
            Box::new(EmbedDatabaseStage),
            Box::new(WriteDeployRecordStage),
            Box::new(StageBootStage),
        ]);

        let result = if orchestrator.validate(&context).is_err() {
            Err((GenesisStage::Setup, SetupError::from(CommonError::PipelineInvalid)))
        } else {
            orchestrator
                .run_exclusive(&mut context, self.cancel_token)
                .map_err(|failure| match failure {
                    OrchestratorError::Setup(lock_error) => (GenesisStage::Setup, SetupError::from(lock_error)),
                    OrchestratorError::Stage(index, error) => (GenesisStage::from_stage_index(index), error),
                })
        };

        self.cancel_token.reset();

        result
    }
}

impl SetupWholeDiskData<'_> {
    pub fn run(&self) -> Result<(), (GenesisStage, SetupError)> {
        let target = TargetSysroot::create_whole_disk(self).map_err(|error| (GenesisStage::Setup, error))?;

        let mut context = Context::new();
        context.put(Box::new(Message::new(self.hook_message, self.hook_message_context)) as Box<dyn MessageHook>);
        context.put(target);
        context.put(GenesisInput::from(self));

        let orchestrator = SequentialOrchestrator::new(vec![
            Box::new(PrepareSourceStage),
            Box::new(ReadMetaStage),
            Box::new(ImportTreesStage),
            Box::new(CreateDatabaseStage),
            Box::new(InsertFileEntryStage),
            Box::new(EmbedDatabaseStage),
            Box::new(WriteDeployRecordStage),
            Box::new(StageBootStage),
        ]);

        let result = if orchestrator.validate(&context).is_err() {
            Err((GenesisStage::Setup, SetupError::from(CommonError::PipelineInvalid)))
        } else {
            orchestrator
                .run_exclusive(&mut context, self.cancel_token)
                .map_err(|failure| match failure {
                    OrchestratorError::Setup(lock_error) => (GenesisStage::Setup, SetupError::from(lock_error)),
                    OrchestratorError::Stage(index, error) => (GenesisStage::from_stage_index(index), error),
                })
        };

        self.cancel_token.reset();

        result
    }
}
