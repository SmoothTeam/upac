// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsStr;
use std::path::Path;
use std::str::from_utf8;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tempfile::NamedTempFile;

use xattr::FileExt;

use upac_composefs::Digest;

use super::deployment::SystemDeploy;
use super::error::DeployError;
use super::layout::snapshot::{DIGEST_XATTR, MESSAGE_XATTR};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigSnapshot {
    timestamp: u128,
    digest: Digest,
    message: Option<String>,
}

impl ConfigSnapshot {
    pub fn new(digest: Digest, message: Option<String>) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_nanos(),
            digest,
            message,
        }
    }

    pub fn timestamp(&self) -> u128 {
        self.timestamp
    }

    pub fn digest(&self) -> &Digest {
        &self.digest
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub(crate) fn read(deploy: &SystemDeploy, record_name: &OsStr) -> Result<Self, DeployError> {
        fn xattr_text(record: &Path, name: &str) -> Result<Option<String>, DeployError> {
            xattr::get(record, name)?
                .map(|value| {
                    from_utf8(&value)
                        .map(str::to_owned)
                        .map_err(|_| DeployError::InvalidSnapshot)
                })
                .transpose()
        }

        let record = deploy.history_dir().join(record_name);

        let timestamp = record_name
            .to_str()
            .and_then(|record_name| record_name.parse().ok())
            .ok_or(DeployError::InvalidSnapshot)?;

        let digest = xattr_text(&record, DIGEST_XATTR)?.ok_or(DeployError::InvalidSnapshot)?;

        Ok(Self {
            timestamp,
            digest: Digest::from_hex(&digest).map_err(|_| DeployError::InvalidSnapshot)?,
            message: xattr_text(&record, MESSAGE_XATTR)?,
        })
    }

    pub(crate) fn write(&self, deploy: &SystemDeploy) -> Result<(), DeployError> {
        let record = NamedTempFile::new_in(deploy.dir())?;
        record
            .as_file()
            .set_xattr(DIGEST_XATTR, self.digest.to_hex().as_bytes())?;

        if let Some(message) = &self.message {
            record.as_file().set_xattr(MESSAGE_XATTR, message.as_bytes())?;
        }

        record
            .persist_noclobber(deploy.history_dir().join(self.timestamp.to_string()))
            .map_err(|error| error.error)?;

        Ok(())
    }
}
