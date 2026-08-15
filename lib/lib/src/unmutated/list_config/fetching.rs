// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::database::record::DeployRecord;
use crate::deploy::digest::current_prefix_digest;
use crate::deploy::{Deploy, DeployMode};
use crate::errors::CommonError;
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage};
use crate::types::{ConfigCommitEntry, RequestedPrefixDigest};
use crate::unmutated::list_config::ListConfigError;

pub struct FetchingStage;

impl Stage<ListConfigError> for FetchingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), ListConfigError> {
        let requested = context
            .get::<RequestedPrefixDigest>()
            .ok_or(CommonError::MissingResult)?;

        let prefix_digest = match &requested.0 {
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

        Ok((progress, Box::new(NoRollback)))
    }
}
