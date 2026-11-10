// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::CVersion;
use upac_abi::response::entry::{
    CConfigSnapshotEntry, CDeployEntry, CDiffFileEntry, CDiffPackageEntry, CSearchFileEntry, CTransactionEntry,
};
use upac_abi::types::{COwned, CSlice, CVec};

use upac_macro::{CTryToRust, RustToC};

use crate::error::ErrorKind;
use crate::transaction::TransactionKind;
use crate::{FileDiffKind, PackageDiffKind, Version};

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

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct TransactionEntry {
    pub timestamp: u64,
    pub kind: TransactionKind,
    pub subject: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct ConfigSnapshotEntry {
    pub timestamp: u64,
    pub digest: String,
    pub message: Option<String>,
}
