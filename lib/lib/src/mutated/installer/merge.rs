// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs::create_dir_all;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use composefs::fsverity::FsVerityHashValue;
use composefs::repository::ImportContext;

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::composefs::overlay::{apply_overlay_upper, apply_tree_overlay};
use crate::composefs::repository::commit_tree;
use crate::config::merge::merge_config;
use crate::database::error::DeployRecordError;
use crate::database::record::{ConfigHistoryEntry, DeployRecord};
use crate::deploy::Deploy;
use crate::deploy::digest::current_prefix_digest;
use crate::errors::CommonError;
use crate::layout::deployment::ETC_UPPER_RELATIVE_PATH;
use crate::mutated::installer::{CommitMessage, InstallError, NewConfigDefaults, NewPrefixDigest, Subject};
use crate::orchestrator::Context;
use crate::orchestrator::stage::{RollbackGuard, Stage};

pub struct MergeStage;

impl Stage<InstallError> for MergeStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), InstallError> {
        let new_config_defaults = context.take::<NewConfigDefaults>().ok_or(CommonError::MissingResult)?;
        let new_prefix = context.get::<NewPrefixDigest>().ok_or(CommonError::MissingResult)?;
        let deploy = context.get::<Deploy>().ok_or(CommonError::MissingResult)?;
        let subject = context.get::<Subject>().ok_or(CommonError::MissingResult)?;
        let message = context.get::<CommitMessage>().ok_or(CommonError::MissingResult)?;

        let repository = deploy.open_repository()?;

        let current_prefix = current_prefix_digest()?;
        let current_record_dir = deploy.deploy(&current_prefix);
        let current_record = DeployRecord::read(&current_record_dir)?;

        let base = deploy.open_tree(&current_record.working_config)?;

        let mut live = base.clone();
        let etc_upper_dir = current_record_dir.join(ETC_UPPER_RELATIVE_PATH);
        let mut import_ctx = ImportContext::default();
        apply_overlay_upper(&repository, &mut live, &etc_upper_dir, &mut import_ctx)?;

        let mut new = base.clone();
        apply_tree_overlay(&mut new, &new_config_defaults.0)?;

        // The doc calls for notifying the user about `<path>.upac-new` conflicts through the
        // message-hook mechanism — deferred, no established precedent yet for a non-fatal
        // notification of this shape (see also the `up mime sync` best-effort refresh, same gap).
        let merge_result = merge_config(&base, &new, &live)?;
        let new_config_digest = commit_tree(&repository, merge_result.tree)?.to_hex();

        let new_record_dir = deploy.deploy(&new_prefix.0);
        let mut record = match DeployRecord::read(&new_record_dir) {
            Ok(existing) => existing,
            Err(DeployRecordError::NotFound) => {
                create_dir_all(&new_record_dir).map_err(DeployRecordError::from)?;

                DeployRecord {
                    prefix_digest: new_prefix.0.clone(),
                    subject: subject.0.clone(),
                    message: message.0.clone(),
                    seq: DeployRecord::allocate_seq(deploy)?,
                    timestamp: now_secs(),
                    config_history: Vec::new(),
                    // Never equals a real digest — guarantees the shared "did working_config
                    // change" check below always fires for a brand-new record, sealing its first
                    // config_history entry and writing meta.json for the first time.
                    working_config: String::new(),
                }
            }
            Err(error) => return Err(error.into()),
        };

        let mut written = Vec::new();
        if record.working_config != new_config_digest {
            record.working_config = new_config_digest.clone();
            record.config_history.push(ConfigHistoryEntry {
                config_digest: new_config_digest,
                subject: subject.0.clone(),
                message: message.0.clone(),
            });
            written.push(record.write(&new_record_dir)?);
        }

        Ok((progress, Box::new(written)))
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}
