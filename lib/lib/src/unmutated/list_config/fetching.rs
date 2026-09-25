// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::CancelToken;

use upac_types::RequestedPrefixDigest;
use upac_types::hook::ProgressEventBuilder;
use upac_types::response::entry::ConfigCommitEntry;

use upac_deploy::digest::current_prefix_digest;
use upac_deploy::record::DeployRecord;
use upac_deploy::{Deploy, DeployMode};

use upac_orchestrator::context::{Context, ctx_get};
use upac_orchestrator::stage::{NoRollback, RollbackGuard, Stage, StageResult};

use super::ListConfigError;

pub struct FetchingStage;

impl Stage<ListConfigError> for FetchingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), ListConfigError> {
        let requested = ctx_get!(context, RequestedPrefixDigest);

        let prefix_digest = match &**requested {
            Some(prefix_digest) => prefix_digest.clone(),
            None => current_prefix_digest()?,
        };

        let deploy = Deploy::new(DeployMode::ReadOnly)?;
        let record = DeployRecord::read(&deploy.deploy(&prefix_digest))?;

        let entries: Vec<ConfigCommitEntry> = record
            .config_history
            .into_iter()
            .map(|entry| ConfigCommitEntry {
                config_digest: entry.config_digest,
                subject: entry.subject,
                message: entry.message,
            })
            .collect();

        context.put(entries);

        Ok((progress, StageResult::Advance, Box::new(NoRollback)))
    }
}
