// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, read_to_string};
use std::io::{ErrorKind as IoErrorKind, Read};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use upac_macro::JsonCodec;

use crate::database::error::{ConfigDigestResolveError, DeployRecordError, DeployRecordsError};
use crate::deploy::Deploy;
use crate::deploy::digest::current_prefix_digest;
use crate::fs::WrittenFile;
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
    pub pinned: bool,
}

impl DeployRecord {
    pub fn read(deploy_dir: &Path) -> Result<Self, DeployRecordError> {
        let mut content = String::new();
        File::open(deploy_dir.join(RECORD_FILENAME))?.read_to_string(&mut content)?;

        let value: serde_json::Value = serde_json::from_str(&content)?;

        Self::from_json(&value)
    }

    pub fn write(&self, deploy_dir: &Path) -> Result<WrittenFile, DeployRecordError> {
        let content = serde_json::to_vec_pretty(&self.to_json())?;

        Ok(WrittenFile::write(&deploy_dir.join(RECORD_FILENAME), &content)?)
    }

    pub fn update_working_config(
        &mut self, deploy_dir: &Path, new_config_digest: String, subject: String, message: Option<String>,
    ) -> Result<Option<WrittenFile>, DeployRecordError> {
        if self.working_config == new_config_digest {
            return Ok(None);
        }

        self.working_config = new_config_digest.clone();
        self.config_history.push(ConfigHistoryEntry {
            config_digest: new_config_digest,
            subject,
            message,
        });

        Ok(Some(self.write(deploy_dir)?))
    }

    pub fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs()
    }

    pub fn allocate_seq(next_seq_path: &Path) -> Result<u64, DeployRecordError> {
        let current = match read_to_string(next_seq_path) {
            Ok(content) => content.trim().parse().map_err(|_| DeployRecordError::InvalidField)?,
            Err(error) if error.kind() == IoErrorKind::NotFound => 0,
            Err(error) => return Err(error.into()),
        };

        WrittenFile::write(next_seq_path, (current + 1).to_string().as_bytes())?;

        Ok(current)
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
