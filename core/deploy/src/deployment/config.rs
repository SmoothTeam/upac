// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use upac_composefs::Digest;

use super::Deployment;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigDeploy {
    digest: Digest,
    subject: String,
    message: Option<String>,
    timestamp: u64,
}

impl ConfigDeploy {
    pub fn new(digest: Digest, subject: String, message: Option<String>) -> Self {
        Self {
            digest,
            subject,
            message,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs(),
        }
    }
}

impl Deployment for ConfigDeploy {
    fn digest(&self) -> &Digest {
        &self.digest
    }

    fn subject(&self) -> &str {
        &self.subject
    }

    fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    fn timestamp(&self) -> u64 {
        self.timestamp
    }
}
