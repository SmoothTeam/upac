// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionKind {
    Bootstrap,
    Install,
    Update,
    Uninstall,
    Files,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub uuid: Uuid,
    pub parent: Option<String>,
    pub timestamp: u64,
    pub kind: TransactionKind,
    pub subject: String,
    pub message: Option<String>,
}

impl Transaction {
    pub fn new(parent: Option<String>, kind: TransactionKind, subject: String, message: Option<String>) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            parent,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs(),
            kind,
            subject,
            message,
        }
    }
}
