// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::create_dir_all;

use composefs::fsverity::FsVerityHashValue;

use upac::database::record::DeployRecord;
use upac::fs::WrittenFile;
use upac::orchestrator::context::{Context, ctx_get};
use upac::orchestrator::stage::{RollbackGuard, Stage, StageResult};

use upac_abi::hook::CancelToken;

use upac_types::hook::ProgressEventBuilder;

use super::error::BootstrapError;
use super::{DeployDigests, Pinned};

use crate::target::TargetSysroot;

#[cfg(test)]
#[path = "../../../tests/inline/deploy.rs"]
mod tests;

pub struct WriteDeployRecordStage;

impl Stage<BootstrapError> for WriteDeployRecordStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), BootstrapError> {
        let deploy_digest = ctx_get!(context, DeployDigests);
        let target = ctx_get!(context, TargetSysroot);
        let pinned = ctx_get!(context, Pinned);

        let deploy_dir = target.deploy_dir(&deploy_digest.prefix.to_hex());
        create_dir_all(&deploy_dir)?;

        let record = DeployRecord {
            prefix_digest: deploy_digest.prefix.to_hex(),
            subject: "genesis".to_owned(),
            message: None,
            seq: DeployRecord::allocate_seq(&target.next_seq_path())?,
            timestamp: DeployRecord::now_secs(),
            config_history: Vec::new(),
            working_config: deploy_digest.config.to_hex(),
            pinned: **pinned,
        };
        let written_file = record.write(&deploy_dir)?;

        let guard: Box<dyn RollbackGuard> = Box::new(vec![written_file] as Vec<WrittenFile>);

        Ok((progress, StageResult::Advance, guard))
    }
}
