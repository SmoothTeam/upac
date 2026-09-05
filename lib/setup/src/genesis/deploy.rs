// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::create_dir_all;

use composefs::fsverity::FsVerityHashValue;

use upac::database::record::DeployRecord;
use upac::fs::WrittenFile;
use upac::orchestrator::Context;
use upac::orchestrator::stage::{RollbackGuard, Stage, StageResult};

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use super::ctx_get;

use crate::error::SetupError;
use crate::target::TargetSysroot;
use crate::types::{ConfigDigest, GenesisInput, PrefixDigest};

#[cfg(test)]
#[path = "../../tests/inline/deploy.rs"]
mod tests;

pub struct WriteDeployRecordStage;

impl Stage<SetupError> for WriteDeployRecordStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), SetupError> {
        let target = ctx_get!(context, TargetSysroot);
        let input = ctx_get!(context, GenesisInput);
        let prefix_digest = ctx_get!(context, PrefixDigest);
        let config_digest = ctx_get!(context, ConfigDigest);

        let prefix_digest_hex = prefix_digest.0.to_hex();
        let deploy_dir = target.deploy_dir(&prefix_digest_hex);
        create_dir_all(&deploy_dir)?;

        let record = DeployRecord {
            prefix_digest: prefix_digest_hex,
            subject: "genesis".to_owned(),
            message: None,
            seq: DeployRecord::allocate_seq(&target.next_seq_path())?,
            timestamp: DeployRecord::now_secs(),
            config_history: Vec::new(),
            working_config: config_digest.0.to_hex(),
            pinned: input.pinned,
        };
        let written_file = record.write(&deploy_dir)?;

        let guard: Box<dyn RollbackGuard> = Box::new(vec![written_file] as Vec<WrittenFile>);

        Ok((progress, StageResult::Advance, guard))
    }
}
