// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::collections::HashSet;
use std::fs::remove_dir_all;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use upac_types::settings::RuntimeSettings;

use crate::composefs::error::RepoError;
use crate::database::record::DeployRecord;
use crate::deploy::Deploy;
use crate::deploy::digest::current_prefix_digest;
use crate::errors::CommonError;
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage};

impl Deploy {
    pub fn prune_deploys(&self) -> Result<Vec<String>, CommonError> {
        let retention_depth = RuntimeSettings::load().gc.retention_depth as usize;

        let mut records = DeployRecord::read_all(self)?;
        records.sort_by_key(|record| std::cmp::Reverse(record.seq));

        let mut pinned: HashSet<String> = HashSet::new();

        if let Ok(current) = current_prefix_digest()
            && let Some(current_record) = records.iter().find(|record| record.prefix_digest == current)
        {
            pinned.insert(current_record.prefix_digest.clone());

            if let Some(previous) = records.iter().find(|record| record.seq < current_record.seq) {
                pinned.insert(previous.prefix_digest.clone());
            }
        }

        for record in &records {
            if record.pinned {
                pinned.insert(record.prefix_digest.clone());
            }
        }

        for record in records.iter().take(retention_depth) {
            pinned.insert(record.prefix_digest.clone());
        }

        let mut removed = Vec::new();
        for record in &records {
            if pinned.contains(&record.prefix_digest) {
                continue;
            }

            remove_dir_all(self.deploy(&record.prefix_digest)).map_err(RepoError::from)?;
            removed.push(record.prefix_digest.clone());
        }

        Ok(removed)
    }
}

pub struct RetentionStage;

impl<E: From<CommonError> + Send + 'static> Stage<E> for RetentionStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), E> {
        let deploy = context.get::<Deploy>().ok_or(CommonError::MissingResult)?;
        deploy.prune_deploys()?;

        Ok((progress, Box::new(NoRollback)))
    }
}
