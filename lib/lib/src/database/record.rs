// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs::{File, rename};
use std::io::Read;
use std::path::Path;

use upac_macro::JsonCodec;

use crate::database::error::DeployRecordError;
use crate::types::deployment::RECORD_FILENAME;

#[derive(Debug, Clone, PartialEq, Eq, JsonCodec)]
pub struct EtcHistoryEntry {
    pub etc_digest: String,
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
    pub etc_history: Vec<EtcHistoryEntry>,
    pub working_etc: String,
}

impl DeployRecord {
    pub fn read(deploy_dir: &Path) -> Result<Self, DeployRecordError> {
        let mut content = String::new();
        File::open(deploy_dir.join(RECORD_FILENAME))?.read_to_string(&mut content)?;

        let value: serde_json::Value = serde_json::from_str(&content)?;

        Self::from_json(&value)
    }

    pub fn write(&self, deploy_dir: &Path) -> Result<(), DeployRecordError> {
        let tmp_path = deploy_dir.join(format!(".{RECORD_FILENAME}.tmp"));

        let mut tmp_file = File::create(&tmp_path)?;
        serde_json::to_writer_pretty(&mut tmp_file, &self.to_json())?;
        tmp_file.sync_all()?;

        rename(&tmp_path, deploy_dir.join(RECORD_FILENAME))?;

        Ok(())
    }
}
