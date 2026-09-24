// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac::orchestrator::context::{Context, ctx_get};
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;

use super::error::FormatError;
use super::filesystem::FormatTarget;
use super::{RequestedDevice, RequestedFilesystem};

pub struct MkfsStage;

impl Stage<FormatError> for MkfsStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), FormatError> {
        let device_path = ctx_get!(context, RequestedDevice);
        let filesystem = ctx_get!(context, RequestedFilesystem);

        FormatTarget {
            device_path,
            label: filesystem.label.as_deref(),
        }
        .format(
            filesystem.fs_kind,
            filesystem.btrfs_node_size,
            filesystem.btrfs_sector_size,
        )?;

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
