// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::entry::{
    CConfigSnapshotEntry, CDeployEntry, CDiffFileEntry, CDiffPackageEntry, CSearchFileEntry, CTransactionEntry,
};
use upac_abi::request::CRequestBase;
use upac_abi::request::unmutated::{
    CDiffRequest, CListConfigRequest, CListDeploysRequest, CListHistoryRequest, CListPackagesRequest,
    CSearchFilesRequest, CSearchInMetaRequest, CSearchInPackageFilesRequest, CSearchMetaRequest,
};
use upac_abi::types::{COwned, CSlice, CVec};
use upac_abi::{CPackageId, CPackageMeta};

use upac_macro::{CTryToRust, RustToC};

use super::RequestBase;

use crate::entry::{
    ConfigSnapshotEntry, DeployEntry, DiffFileEntry, DiffPackageEntry, SearchFileEntry, TransactionEntry,
};
use crate::error::ErrorKind;
use crate::package::{PackageId, PackageMeta};

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct ListPackagesRequest<'data> {
    pub base: RequestBase<'data>,

    #[out]
    pub metas: Vec<PackageMeta>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct ListDeploysRequest<'data> {
    pub base: RequestBase<'data>,

    #[out]
    pub deploys: Vec<DeployEntry>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct ListHistoryRequest<'data> {
    pub base: RequestBase<'data>,
    pub digest: Option<&'data str>,

    #[out]
    pub transactions: Vec<TransactionEntry>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct ListConfigRequest<'data> {
    pub base: RequestBase<'data>,
    pub digest: Option<&'data str>,

    #[out]
    pub snapshots: Vec<ConfigSnapshotEntry>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct DiffRequest<'data> {
    pub base: RequestBase<'data>,
    pub from_digest: Option<&'data str>,
    pub to_digest: Option<&'data str>,

    #[out]
    pub packages: Vec<DiffPackageEntry>,
    #[out]
    pub files: Vec<DiffFileEntry>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SearchMetaRequest<'data> {
    pub base: RequestBase<'data>,
    pub search: &'data str,
    pub is_regex: bool,

    #[out]
    pub metas: Vec<PackageMeta>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SearchFilesRequest<'data> {
    pub base: RequestBase<'data>,
    pub search: &'data str,
    pub is_regex: bool,

    #[out]
    pub files: Vec<SearchFileEntry>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SearchInMetaRequest<'data> {
    pub base: RequestBase<'data>,
    pub package: PackageId,
    pub search: &'data str,
    pub is_regex: bool,

    #[out]
    pub metas: Vec<PackageMeta>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SearchInPackageFilesRequest<'data> {
    pub base: RequestBase<'data>,
    pub package: PackageId,
    pub search: &'data str,
    pub is_regex: bool,

    #[out]
    pub files: Vec<SearchFileEntry>,
}
