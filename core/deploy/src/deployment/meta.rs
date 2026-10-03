// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::read as read_file;
use std::path::Path;

use serde::{Deserialize, Serialize};

use upac_composefs::Digest;
use upac_composefs::fs::WrittenFile;

use super::super::error::PrefixMetaError;
use super::super::layout::deployment::PREFIX_META_FILENAME;
use super::config::ConfigDeploy;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PrefixMetaState {
    pub(super) pinned: bool,
    pub(super) current_config: usize,
    pub(super) configs: Vec<ConfigDeploy>,
}

impl PrefixMetaState {
    pub(super) fn read(deploy_dir: &Path) -> Result<Self, PrefixMetaError> {
        let content = read_file(deploy_dir.join(PREFIX_META_FILENAME))?;

        Ok(serde_json::from_slice(&content)?)
    }

    pub(super) fn write(&self, deploy_dir: &Path) -> Result<WrittenFile, PrefixMetaError> {
        let content = serde_json::to_vec_pretty(self)?;

        Ok(WrittenFile::write(&deploy_dir.join(PREFIX_META_FILENAME), &content)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PrefixPointer {
    pub(crate) prefix_digest: Digest,
}

impl PrefixPointer {
    pub(crate) fn read(path: &Path) -> Result<Self, PrefixMetaError> {
        let content = read_file(path)?;

        Ok(serde_json::from_slice(&content)?)
    }

    pub(crate) fn write(&self, path: &Path) -> Result<WrittenFile, PrefixMetaError> {
        let content = serde_json::to_vec_pretty(self)?;

        Ok(WrittenFile::write(path, &content)?)
    }
}
