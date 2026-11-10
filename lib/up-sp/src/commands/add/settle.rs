// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::thread::sleep;
use std::time::Duration;

use upac_types::CancelToken;
use upac_types::error::ErrorKind;
use upac_types::response::partition::SetupPartitionAddResponse;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::RequestedPartition;

use crate::gpt::Partition;
use crate::layout::partition::{SETTLE_ATTEMPTS, SETTLE_INTERVAL_MS};

pub struct SettleStage;

#[stage]
impl Stage<ErrorKind> for SettleStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let partition = context.take::<Partition>()?;
        let requested = context.take::<RequestedPartition>()?;

        let mut attempts_left = SETTLE_ATTEMPTS;
        while !partition.path().exists() {
            if attempts_left == 0 {
                return Err(ErrorKind::NotFound);
            }
            if cancel.is_cancelled() {
                return Err(ErrorKind::Cancelled);
            }

            sleep(Duration::from_millis(u64::from(SETTLE_INTERVAL_MS)));
            attempts_left -= 1;
        }

        context.put(SetupPartitionAddResponse {
            label: requested.label,
            device_path: partition.path().to_string_lossy().into_owned(),
        });

        Ok(())
    }
}
