// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::package::CPackageMeta;
use upac_abi::response::entry::{
    CConfigCommitEntry, CDiffConfigFileEntry, CDiffPackageEntry, CDiffPrefixFileEntry, CDiffUntrackedFileEntry,
    CHistoryEntry, CPrefixEntry, CSearchFileEntry,
};
use upac_abi::response::unmutated::{
    CDiffConfigResponse, CDiffPackagesResponse, CDiffPrefixResponse, CDiffResponse, CListConfigResponse,
    CListHistoryResponse, CListPackagesResponse, CListPrefixResponse, CSearchFilesResponse, CSearchInMetaResponse,
    CSearchInPackageFilesResponse, CSearchMetaResponse,
};
use upac_abi::types::{COwned, CVec};

use upac_macro::RustToC;

use super::entry::{
    ConfigCommitEntry, DiffConfigFileEntry, DiffPackageEntry, DiffPrefixFileEntry, DiffUntrackedFileEntry,
    HistoryEntry, PrefixEntry, SearchFileEntry,
};

use crate::package::PackageMeta;

#[derive(Debug, Clone, RustToC)]
pub struct DiffPrefixResponse {
    pub files: Vec<DiffPrefixFileEntry>,
}

#[derive(Debug, Clone, RustToC)]
pub struct DiffConfigResponse {
    pub files: Vec<DiffConfigFileEntry>,
}

#[derive(Debug, Clone, RustToC)]
pub struct DiffPackagesResponse {
    pub diff_packages: Vec<DiffPackageEntry>,
}

#[derive(Debug, Clone, RustToC)]
pub struct DiffResponse {
    pub diff_packages: Vec<DiffPackageEntry>,
    pub unattached_files: Vec<DiffUntrackedFileEntry>,
}

#[derive(Debug, Clone, RustToC)]
pub struct ListPrefixResponse {
    pub prefixes: Vec<PrefixEntry>,
}

#[derive(Debug, Clone, RustToC)]
pub struct ListHistoryResponse {
    pub history: Vec<HistoryEntry>,
}

#[derive(Debug, Clone, RustToC)]
pub struct ListConfigResponse {
    pub commits: Vec<ConfigCommitEntry>,
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
