// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::ErrorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionKind {
    Bootstrap,
    Install,
    Update,
    Uninstall,
    Files,
}

impl TransactionKind {
    pub const ALL: [TransactionKind; 5] = [
        TransactionKind::Bootstrap,
        TransactionKind::Install,
        TransactionKind::Update,
        TransactionKind::Uninstall,
        TransactionKind::Files,
    ];
}

impl Display for TransactionKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_str(match self {
            TransactionKind::Bootstrap => "bootstrap",
            TransactionKind::Install => "install",
            TransactionKind::Update => "update",
            TransactionKind::Uninstall => "uninstall",
            TransactionKind::Files => "files",
        })
    }
}

impl FromStr for TransactionKind {
    type Err = ErrorKind;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        TransactionKind::ALL
            .into_iter()
            .find(|kind| kind.to_string() == name)
            .ok_or(ErrorKind::InvalidEntry)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub timestamp: u128,
    pub kind: TransactionKind,
    pub subject: String,
    pub message: Option<String>,
}

impl Transaction {
    pub fn new(kind: TransactionKind, subject: String, message: Option<String>) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_nanos(),
            kind,
            subject,
            message,
        }
    }
}
