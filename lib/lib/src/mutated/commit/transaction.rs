// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use composefs::fsverity::FsVerityHashValue;
use composefs::repository::ImportContext;

use upac_abi::hook::CancelToken;
use upac_types::hook::ProgressEventBuilder;

use upac_composefs::overlay::apply_overlay_upper;
use upac_composefs::repository::commit_tree;
use upac_deploy::Deploy;
use upac_deploy::digest::current_prefix_digest;
use upac_deploy::layout::deployment::CONFIG_DIR_NAME;
use upac_deploy::record::DeployRecord;
use upac_orchestrator::context::{Context, ctx_get};
use upac_orchestrator::stage::{RollbackGuard, Stage, StageResult};

use super::{CommitError, CommitInfo};

pub struct TransactionStage;

impl Stage<CommitError> for TransactionStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, StageResult, Box<dyn RollbackGuard>), CommitError> {
        let deploy = ctx_get!(context, Deploy);
        let commit_info = ctx_get!(context, CommitInfo);

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
            commit_info.subject.clone(),
            commit_info.message.clone(),
        )?);

        Ok((progress, StageResult::Advance, Box::new(written)))
    }
}
