// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs::File;
use std::io::Read;
use std::path::Path;

use upac_macro::JsonCodec;

use crate::database::error::{ConfigDigestResolveError, DeployRecordError, DeployRecordsError};
use crate::deploy::Deploy;
use crate::deploy::digest::current_prefix_digest;
use crate::fs::atomic_write;
use crate::layout::deployment::RECORD_FILENAME;

#[derive(Debug, Clone, PartialEq, Eq, JsonCodec)]
pub struct ConfigHistoryEntry {
    pub config_digest: String,
    pub subject: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, JsonCodec)]
pub struct DeployRecord {
    pub prefix_digest: String,
    pub subject: String,
    pub message: Option<String>,
    pub seq: u64,
    pub timestamp: u64,
    pub config_history: Vec<ConfigHistoryEntry>,
    pub working_config: String,
}

impl DeployRecord {
    pub fn read(deploy_dir: &Path) -> Result<Self, DeployRecordError> {
        let mut content = String::new();
        File::open(deploy_dir.join(RECORD_FILENAME))?.read_to_string(&mut content)?;

        let value: serde_json::Value = serde_json::from_str(&content)?;

        Self::from_json(&value)
    }

    pub fn write(&self, deploy_dir: &Path) -> Result<(), DeployRecordError> {
        let content = serde_json::to_vec_pretty(&self.to_json())?;
        atomic_write(&deploy_dir.join(RECORD_FILENAME), &content)?;

        Ok(())
    }

    pub fn read_all(deploy: &Deploy) -> Result<Vec<DeployRecord>, DeployRecordsError> {
        let mut records = Vec::new();

        for prefix_digest in deploy.deploys()? {
            match Self::read(&deploy.deploy(&prefix_digest)) {
                Ok(record) => records.push(record),
                Err(DeployRecordError::NotFound) => continue,
                Err(error) => return Err(error.into()),
            }
        }

        Ok(records)
    }

    pub fn resolve_config_digest(
        deploy: &Deploy, requested: Option<&str>,
    ) -> Result<(String, String), ConfigDigestResolveError> {
        match requested {
            Some(config_digest) => Self::resolve_requested(deploy, config_digest),
            None => Self::resolve_current(deploy).map_err(ConfigDigestResolveError::from),
        }
    }

    fn resolve_requested(deploy: &Deploy, config_digest: &str) -> Result<(String, String), ConfigDigestResolveError> {
        Self::read_all(deploy)?
            .into_iter()
            .find(|record| record.owns_config_digest(config_digest))
            .map(|record| (config_digest.to_owned(), record.prefix_digest))
            .ok_or_else(|| ConfigDigestResolveError::NotFound(config_digest.to_owned()))
    }

    fn owns_config_digest(&self, config_digest: &str) -> bool {
        self.working_config == config_digest
            || self
                .config_history
                .iter()
                .any(|entry| entry.config_digest == config_digest)
    }

    pub fn resolve_own_config_digest(&self, requested: Option<&str>) -> Option<String> {
        match requested {
            Some(config_digest) => self.owns_config_digest(config_digest).then(|| config_digest.to_owned()),
            None => Some(self.working_config.clone()),
        }
    }

    fn resolve_current(deploy: &Deploy) -> Result<(String, String), DeployRecordsError> {
        let prefix_digest = current_prefix_digest()?;
        let record = Self::read(&deploy.deploy(&prefix_digest))?;

        Ok((record.working_config, prefix_digest))
    }
}
