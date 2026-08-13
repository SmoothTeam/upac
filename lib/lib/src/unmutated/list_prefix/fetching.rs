// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::database::error::DeployRecordError;
use crate::database::record::DeployRecord;
use crate::deploy::{Deploy, DeployMode};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage};
use crate::types::PrefixEntry;
use crate::unmutated::list_prefix::ListPrefixError;

pub struct FetchingStage;

impl Stage<ListPrefixError> for FetchingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), ListPrefixError> {
        let deploy = Deploy::new(DeployMode::ReadOnly)?;

        let mut entries = Vec::new();

        for prefix_digest in deploy.deploys()? {
            let record = match DeployRecord::read(&deploy.deploy(&prefix_digest)) {
                Ok(record) => record,
                Err(DeployRecordError::NotFound) => continue,
                Err(error) => return Err(error.into()),
            };

            entries.push(PrefixEntry {
                prefix_digest: record.prefix_digest,
                subject: record.subject,
                message: record.message,
                timestamp: record.timestamp,
                working_config: Some(record.working_etc),
            });
        }

        context.put(entries);

        Ok((progress, Box::new(NoRollback)))
    }
}
