// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::database::record::DeployRecord;
use crate::deploy::Deploy;
use crate::errors::CommonError;
use crate::mutated::rollback::{RequestedConfigDigest, RollbackError, TargetPrefixDigest};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{RollbackGuard, Stage};

pub struct MergeStage;

impl Stage<RollbackError> for MergeStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), RollbackError> {
        let requested = context
            .get::<RequestedConfigDigest>()
            .ok_or(CommonError::MissingResult)?;
        let deploy = context.get::<Deploy>().ok_or(CommonError::MissingResult)?;

        let (config_digest, prefix_digest) = DeployRecord::resolve_config_digest(deploy, Some(&requested.0))?;

        let record_dir = deploy.deploy(&prefix_digest);
        let mut record = DeployRecord::read(&record_dir)?;

        let mut written = Vec::new();
        if record.working_config != config_digest {
            record.working_config = config_digest;
            written.push(record.write(&record_dir)?);
        }

        context.put(TargetPrefixDigest(prefix_digest));

        Ok((progress, Box::new(written)))
    }
}
