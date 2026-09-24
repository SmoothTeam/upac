// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac::orchestrator::context::{Context, ctx_take};
use upac::orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use upac_abi::FsKind;
use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;

use super::RequestedMount;
use super::error::BootstrapError;

use crate::target::TargetSysroot;

pub struct MountStage;

impl Stage<BootstrapError> for MountStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), BootstrapError> {
        let requested = ctx_take!(context, RequestedMount);

        if requested.deploy_fs == FsKind::Vfat {
            return Err(BootstrapError::UnsupportedDeployFs);
        }

        let target = TargetSysroot::new(
            &requested.deploy_device,
            requested.deploy_fs,
            &requested.esp_device,
            requested.mount_point,
        )?;

        context.put(target);

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
