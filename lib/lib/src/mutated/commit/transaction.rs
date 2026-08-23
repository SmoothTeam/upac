// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use composefs::fsverity::FsVerityHashValue;
use composefs::repository::ImportContext;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::composefs::overlay::apply_overlay_upper;
use crate::composefs::repository::commit_tree;
use crate::database::record::{ConfigHistoryEntry, DeployRecord};
use crate::deploy::Deploy;
use crate::deploy::digest::current_prefix_digest;
use crate::errors::CommonError;
use crate::layout::deployment::ETC_UPPER_RELATIVE_PATH;
use crate::mutated::commit::{CommitError, CommitMessage, Subject};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{RollbackGuard, Stage};

pub struct TransactionStage;

impl Stage<CommitError> for TransactionStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), CommitError> {
        let deploy = context.get::<Deploy>().ok_or(CommonError::MissingResult)?;
        let subject = context.get::<Subject>().ok_or(CommonError::MissingResult)?;
        let message = context.get::<CommitMessage>().ok_or(CommonError::MissingResult)?;

        let repository = deploy.open_repository()?;

        let current_prefix = current_prefix_digest()?;
        let current_record_dir = deploy.deploy(&current_prefix);
        let mut record = DeployRecord::read(&current_record_dir)?;

        let base = deploy.open_tree(&record.working_config)?;

        let mut live = base.clone();
        let etc_upper_dir = current_record_dir.join(ETC_UPPER_RELATIVE_PATH);
        let mut import_ctx = ImportContext::default();
        apply_overlay_upper(&repository, &mut live, &etc_upper_dir, &mut import_ctx)?;

        let new_config_digest = commit_tree(&repository, live)?.to_hex();

        let mut written = Vec::new();
        if record.working_config != new_config_digest {
            record.working_config = new_config_digest.clone();
            record.config_history.push(ConfigHistoryEntry {
                config_digest: new_config_digest,
                subject: subject.0.clone(),
                message: message.0.clone(),
            });
            written.push(record.write(&current_record_dir)?);
        }

        Ok((progress, Box::new(written)))
    }
}
