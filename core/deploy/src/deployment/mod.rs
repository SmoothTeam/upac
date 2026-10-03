// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::Path;

use upac_composefs::Digest;
use upac_composefs::fs::WrittenFile;

use upac_types::transaction::Transaction;

use super::error::{PrefixDeployError, PrefixMetaError};

use self::config::ConfigDeploy;
use self::meta::PrefixMetaState;

pub mod config;

pub(crate) mod meta;

#[cfg(test)]
#[path = "../../tests/inline/prefix.rs"]
mod tests;

pub trait Deployment {
    fn digest(&self) -> &Digest;
    fn subject(&self) -> &str;
    fn message(&self) -> Option<&str>;
    fn timestamp(&self) -> u64;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixDeploy {
    digest: Digest,
    transaction: Transaction,
    state: PrefixMetaState,
}

impl PrefixDeploy {
    pub fn new(digest: Digest, transaction: Transaction, first_config: ConfigDeploy) -> Self {
        Self {
            digest,
            transaction,
            state: PrefixMetaState {
                pinned: false,
                current_config: 0,
                configs: vec![first_config],
            },
        }
    }

    pub(crate) fn read(digest: Digest, transaction: Transaction, deploy_dir: &Path) -> Result<Self, PrefixMetaError> {
        Ok(Self {
            digest,
            transaction,
            state: PrefixMetaState::read(deploy_dir)?,
        })
    }

    pub(crate) fn write(&self, deploy_dir: &Path) -> Result<WrittenFile, PrefixMetaError> {
        self.state.write(deploy_dir)
    }

    pub fn transaction(&self) -> &Transaction {
        &self.transaction
    }

    pub fn pinned(&self) -> bool {
        self.state.pinned
    }

    pub fn configs(&self) -> &[ConfigDeploy] {
        &self.state.configs
    }

    pub fn current_config(&self) -> Option<&ConfigDeploy> {
        self.state.configs.get(self.state.current_config)
    }

    pub fn config(&self, config_index: usize) -> Option<&ConfigDeploy> {
        self.state.configs.get(config_index)
    }

    pub fn referenced_trees(&self) -> Vec<&Digest> {
        let mut trees = vec![&self.digest];
        trees.extend(self.state.configs.iter().map(|config| config.digest()));

        trees
    }

    pub fn add_config(&mut self, config: ConfigDeploy) {
        self.state.configs.push(config);
        self.state.current_config = self.state.configs.len() - 1;
    }

    pub fn switch_config(&mut self, config_index: usize) -> Result<(), PrefixDeployError> {
        if config_index >= self.state.configs.len() {
            return Err(PrefixDeployError::ConfigNotFound(config_index));
        }

        self.state.current_config = config_index;

        Ok(())
    }

    pub fn set_pinned(&mut self, pinned: bool) {
        self.state.pinned = pinned;
    }
}

impl Deployment for PrefixDeploy {
    fn digest(&self) -> &Digest {
        &self.digest
    }

    fn subject(&self) -> &str {
        &self.transaction.subject
    }

    fn message(&self) -> Option<&str> {
        self.transaction.message.as_deref()
    }

    fn timestamp(&self) -> u64 {
        self.transaction.timestamp
    }
}
