// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::composefs::repository::gc;
use crate::database::record::DeployRecord;
use crate::deploy::Deploy;
use crate::errors::CommonError;
use crate::mutated::gc::GcError;
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage};

pub struct CleaningStage;

impl Stage<GcError> for CleaningStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), GcError> {
        let deploy = context.get::<Deploy>().ok_or(CommonError::MissingResult)?;

        deploy.prune_deploys()?;

        let mut roots = Vec::new();
        for prefix_digest in deploy.deploys()? {
            let record = DeployRecord::read(&deploy.deploy(&prefix_digest))?;

            roots.push(record.prefix_digest);
            if !record.working_config.is_empty() {
                roots.push(record.working_config);
            }
            for entry in record.config_history {
                roots.push(entry.config_digest);
            }
        }

        let repository = deploy.open_repository()?;
        let root_refs: Vec<&str> = roots.iter().map(String::as_str).collect();
        gc(&repository, &root_refs)?;

        Ok((progress, Box::new(NoRollback)))
    }
}
