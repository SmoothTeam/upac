// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::thread::sleep;
use std::time::Duration;

use upac::errors::CommonError;
use upac::orchestrator::context::{Context, ctx_get, ctx_take};
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;
use upac_types::response::partition::SetupPartitionAddResponse;

use super::{InsertedPartitionNumber, RequestedPartition};

use crate::commands::partition::RequestedDevice;
use crate::commands::partition::error::PartitionError;
use crate::commands::partition::gpt::partition_node_path;
use crate::layout::partition::{SETTLE_ATTEMPTS, SETTLE_INTERVAL_MS};

pub struct SettleStage;

impl Stage<PartitionError> for SettleStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), PartitionError> {
        let number = ctx_take!(context, InsertedPartitionNumber);
        let requested = ctx_take!(context, RequestedPartition);
        let device_path = ctx_get!(context, RequestedDevice);

        let node_path = partition_node_path(device_path, *number);

        let mut attempts_left = SETTLE_ATTEMPTS;
        while !node_path.exists() {
            if attempts_left == 0 {
                return Err(PartitionError::PartitionNotReady);
            }
            if cancel.is_cancelled() {
                return Err(PartitionError::from(CommonError::Cancelled));
            }

            sleep(Duration::from_millis(u64::from(SETTLE_INTERVAL_MS)));
            attempts_left -= 1;
        }

        context.put(SetupPartitionAddResponse {
            label: requested.label,
            device_path: node_path.to_string_lossy().into_owned(),
        });

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
