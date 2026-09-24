// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac::orchestrator::context::{Context, ctx_get};
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use upac_abi::FsKind;
use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;

use super::error::FormatError;
use super::{RequestedDevice, RequestedFilesystem, RequireEsp};

use crate::commands::partition::gpt::is_esp_partition;

pub struct VerifyStage;

impl Stage<FormatError> for VerifyStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), FormatError> {
        let require_esp = ctx_get!(context, RequireEsp);

        if !**require_esp {
            return Ok((progress, StageResult::Advance, Box::new(NoRollback)));
        }

        let filesystem = ctx_get!(context, RequestedFilesystem);
        if filesystem.fs_kind != FsKind::Vfat {
            return Err(FormatError::InvalidFormatParams);
        }

        let device_path = ctx_get!(context, RequestedDevice);
        if !is_esp_partition(device_path)? {
            return Err(FormatError::NotEspPartition);
        }

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
