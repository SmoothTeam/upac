// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::OpenOptions;

use upac::orchestrator::context::{Context, ctx_get};
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;

use crate::commands::partition::RequestedDevice;
use crate::commands::partition::error::PartitionError;
use crate::commands::partition::gpt::GptTable;

pub struct WriteTableStage;

impl Stage<PartitionError> for WriteTableStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), PartitionError> {
        let device_path = ctx_get!(context, RequestedDevice);

        let mut device = OpenOptions::new().read(true).write(true).open(&**device_path)?;

        let mut table = GptTable::create(&mut device)?;
        table.write_protective_mbr_into(&mut device)?;
        table.write_into(&mut device)?;

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
