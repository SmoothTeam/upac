// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use upac_abi::CVersion;
use upac_abi::entry::{
    CConfigSnapshotEntry, CDeployEntry, CDiffFileEntry, CDiffPackageEntry, CSearchFileEntry, CTransactionEntry,
};
use upac_abi::types::{COwned, CSlice, CVec};

use upac_macro::{CTryToRust, RustToC};

use crate::error::ErrorKind;
use crate::package::Version;
use crate::transaction::TransactionKind;
use crate::{FileDiffKind, PackageDiffKind};

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct DiffFileEntry {
    pub path: String,
    pub kind: FileDiffKind,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct DiffPackageEntry {
    pub name: String,
    pub kind: PackageDiffKind,
    pub version: Version,

    pub files: Vec<DiffFileEntry>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SearchFileEntry {
    pub path: String,
    pub package_name: String,
    pub is_user: bool,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct DeployEntry {
    pub digest: String,
    pub pinned: bool,
    pub current: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, RustToC, CTryToRust)]
pub struct TransactionEntry {
    pub timestamp: u128,
    pub kind: TransactionKind,
    pub subject: String,
    pub message: Option<String>,
}

impl TransactionEntry {
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

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct ConfigSnapshotEntry {
    pub timestamp: u64,
    pub digest: String,
    pub message: Option<String>,
}
