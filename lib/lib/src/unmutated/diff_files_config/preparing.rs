// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_abi::hook::{CancelToken, ProgressEventBuilder};

use crate::composefs::diff::TreeDiff;
use crate::composefs::file::FileHandle;
use crate::database::error::DeployRecordError;
use crate::database::record::DeployRecord;
use crate::database::{InMemory, MemoryDatabase};
use crate::deploy::digest::current_prefix_digest;
use crate::deploy::{Deploy, DeployMode};
use crate::errors::CommonError;
use crate::orchestrator::Context;
use crate::orchestrator::stage::{NoRollback, RollbackGuard, Stage};
use crate::types::RequestedConfigDigestRange;
use crate::types::database::DATABASE_PATH;
use crate::unmutated::diff_files_config::{DiffFilesConfigError, DiffFilesConfigSnapshot};

pub struct PreparingStage;

impl Stage<DiffFilesConfigError> for PreparingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, progress: ProgressEventBuilder,
    ) -> Result<(ProgressEventBuilder, Box<dyn RollbackGuard>), DiffFilesConfigError> {
        let requested = context
            .get::<RequestedConfigDigestRange>()
            .ok_or(CommonError::MissingResult)?;

        let deploy = Deploy::new(DeployMode::ReadOnly)?;

        let (from_config_digest, from_prefix_digest) = Self::resolve(&deploy, requested.from.as_ref())?;
        let (to_config_digest, to_prefix_digest) = Self::resolve(&deploy, requested.to.as_ref())?;

        let repository = deploy.open_repository()?;

        let from_config_tree = deploy.open_tree(&from_config_digest)?;
        let to_config_tree = deploy.open_tree(&to_config_digest)?;

        let changed = TreeDiff::run(&from_config_tree, &to_config_tree);

        let from_prefix_tree = deploy.open_tree(&from_prefix_digest)?;
        let from_bytes = FileHandle::new(DATABASE_PATH).read_file(&repository, &from_prefix_tree)?;
        let from_database = MemoryDatabase::open_in_memory(from_bytes)?;

        let to_prefix_tree = deploy.open_tree(&to_prefix_digest)?;
        let to_bytes = FileHandle::new(DATABASE_PATH).read_file(&repository, &to_prefix_tree)?;
        let to_database = MemoryDatabase::open_in_memory(to_bytes)?;

        context.put(DiffFilesConfigSnapshot {
            changed,
            from_database,
            to_database,
        });

        Ok((progress, Box::new(NoRollback)))
    }
}

impl PreparingStage {
    // Resolves a requested (possibly absent) config_digest to the config_digest
    // itself plus the prefix_digest of the deploy it belongs to — needed because
    // the /usr package database used for file attribution lives under the prefix,
    // not under the standalone /etc image the config_digest names.
    fn resolve(deploy: &Deploy, requested: Option<&String>) -> Result<(String, String), DiffFilesConfigError> {
        match requested {
            Some(config_digest) => {
                for prefix_digest in deploy.deploys()? {
                    let record = match DeployRecord::read(&deploy.deploy(&prefix_digest)) {
                        Ok(record) => record,
                        Err(DeployRecordError::NotFound) => continue,
                        Err(error) => return Err(error.into()),
                    };

                    let owns_config_digest = record.working_etc == *config_digest
                        || record
                            .config_history
                            .iter()
                            .any(|entry| entry.config_digest == *config_digest);

                    if owns_config_digest {
                        return Ok((config_digest.clone(), prefix_digest));
                    }
                }

                Err(DiffFilesConfigError::ConfigDigestNotFound(config_digest.clone()))
            }
            None => {
                let prefix_digest = current_prefix_digest()?;
                let record = DeployRecord::read(&deploy.deploy(&prefix_digest))?;

                Ok((record.working_etc, prefix_digest))
            }
        }
    }
}
