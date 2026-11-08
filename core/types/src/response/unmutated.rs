// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::CPackageMeta;
use upac_abi::response::entry::{
    CConfigSnapshotEntry, CDeployEntry, CDiffFileEntry, CDiffPackageEntry, CSearchFileEntry, CTransactionEntry,
};
use upac_abi::response::unmutated::{
    CDiffResponse, CListConfigResponse, CListDeploysResponse, CListHistoryResponse, CListPackagesResponse,
    CSearchFilesResponse, CSearchInMetaResponse, CSearchInPackageFilesResponse, CSearchMetaResponse,
};
use upac_abi::types::{COwned, CVec};

use upac_macro::RustToC;

use super::entry::{
    ConfigSnapshotEntry, DeployEntry, DiffFileEntry, DiffPackageEntry, SearchFileEntry, TransactionEntry,
};

use crate::PackageMeta;

#[derive(Debug, Clone, RustToC)]
pub struct DiffResponse {
    pub packages: Vec<DiffPackageEntry>,
    pub files: Vec<DiffFileEntry>,
}

#[derive(Debug, Clone, RustToC)]
pub struct ListDeploysResponse {
    pub deploys: Vec<DeployEntry>,
}

#[derive(Debug, Clone, RustToC)]
pub struct ListHistoryResponse {
    pub transactions: Vec<TransactionEntry>,
}

#[derive(Debug, Clone, RustToC)]
pub struct ListConfigResponse {
    pub snapshots: Vec<ConfigSnapshotEntry>,
}

#[derive(Debug, Clone, RustToC)]
pub struct ListPackagesResponse {
    pub metas: Vec<PackageMeta>,
}

#[derive(Debug, Clone, RustToC)]
pub struct SearchMetaResponse {
    pub metas: Vec<PackageMeta>,
}

#[derive(Debug, Clone, RustToC)]
pub struct SearchFilesResponse {
    pub files: Vec<SearchFileEntry>,
}

#[derive(Debug, Clone, RustToC)]
pub struct SearchInMetaResponse {
    pub metas: Vec<PackageMeta>,
}

#[derive(Debug, Clone, RustToC)]
pub struct SearchInPackageFilesResponse {
    pub files: Vec<SearchFileEntry>,
}
