// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use composefs::fsverity::FsVerityHashValue;
use composefs::repository::ImportContext;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::composefs::overlay::apply_overlay_upper;
use crate::composefs::repository::commit_tree;
use crate::database::record::DeployRecord;
use crate::deploy::Deploy;
use crate::deploy::digest::current_prefix_digest;
use crate::layout::deployment::CONFIG_DIR_NAME;
use crate::mutated::commit::{CommitError, CommitMessage, Subject};
use crate::orchestrator::stage::{RollbackGuard, Stage, StageResult};
use crate::orchestrator::{Context, ctx_get};

pub struct TransactionStage;

impl Stage<CommitError> for TransactionStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), CommitError> {
        let deploy = ctx_get!(context, Deploy);
        let subject = ctx_get!(context, Subject);
        let message = ctx_get!(context, CommitMessage);

        let repository = deploy.open_repository()?;

        let current_prefix_name = current_prefix_digest()?;
        let current_record_dir = deploy.deploy(&current_prefix_name);
        let mut record_deploy = DeployRecord::read(&current_record_dir)?;

        let base_config_layout = deploy.open_tree(&record_deploy.working_config)?;

        let mut live_config_layout = base_config_layout.clone();
        let config_upper_dir = current_record_dir.join(CONFIG_DIR_NAME).join("upper");

        let mut import_ctx = ImportContext::default();

        apply_overlay_upper(&repository, &mut live_config_layout, &config_upper_dir, &mut import_ctx)?;

        let new_config_digest = commit_tree(&repository, live_config_layout)?.to_hex();

        let mut written = Vec::new();
        written.extend(record_deploy.update_working_config(
            &current_record_dir,
            new_config_digest,
            subject.0.clone(),
            message.0.clone(),
        )?);

        Ok((progress, StageResult::Advance, Box::new(written)))
    }
}
