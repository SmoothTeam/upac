// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::package::CPackageInfo;
//use upac_abi::request::CRequestBase;
use upac_abi::request::CRequestBase;
use upac_abi::request::unmutated::{
    CDiffConfigRequest, CDiffPackagesRequest, CDiffPrefixRequest, CDiffRequest, CListConfigRequest,
    CListHistoryRequest, CListPackagesRequest, CListPrefixRequest, CSearchFilesRequest, CSearchInMetaRequest,
    CSearchInPackageFilesRequest, CSearchMetaRequest,
};
use upac_abi::types::{COwned, CSlice};

use upac_macro::RustToC;

use super::RequestBase;

use crate::package::PackageInfo;

#[derive(Debug, Clone, RustToC)]
pub struct ListPackagesRequest {
    pub base: RequestBase,
}

#[derive(Debug, Clone, RustToC)]
pub struct ListConfigRequest {
    pub base: RequestBase,
    pub prefix_digest: Option<String>,
}

#[derive(Debug, Clone, RustToC)]
pub struct ListPrefixRequest {
    pub base: RequestBase,
}

#[derive(Debug, Clone, RustToC)]
pub struct ListHistoryRequest {
    pub base: RequestBase,
}

#[derive(Debug, Clone, RustToC)]
pub struct DiffPrefixRequest {
    pub base: RequestBase,
    pub from_prefix_digest: Option<String>,
    pub to_prefix_digest: Option<String>,
}

#[derive(Debug, Clone, RustToC)]
pub struct DiffConfigRequest {
    pub base: RequestBase,
    pub from_config_digest: Option<String>,
    pub to_config_digest: Option<String>,
}

#[derive(Debug, Clone, RustToC)]
pub struct DiffPackagesRequest {
    pub base: RequestBase,
    pub from_prefix_digest: Option<String>,
    pub to_prefix_digest: Option<String>,
}

#[derive(Debug, Clone, RustToC)]
pub struct DiffRequest {
    pub base: RequestBase,
    pub from_prefix_digest: Option<String>,
    pub to_prefix_digest: Option<String>,
    pub from_config_digest: Option<String>,
    pub to_config_digest: Option<String>,
}

#[derive(Debug, Clone, RustToC)]
pub struct SearchMetaRequest {
    pub base: RequestBase,
    pub search: String,
    pub is_regex: bool,
}

#[derive(Debug, Clone, RustToC)]
pub struct SearchFilesRequest {
    pub base: RequestBase,
    pub search: String,
    pub is_regex: bool,
}

#[derive(Debug, Clone, RustToC)]
pub struct SearchInMetaRequest {
    pub base: RequestBase,
    pub package: PackageInfo,
    pub search: String,
    pub is_regex: bool,
}

#[derive(Debug, Clone, RustToC)]
pub struct SearchInPackageFilesRequest {
    pub base: RequestBase,
    pub package: PackageInfo,
    pub search: String,
    pub is_regex: bool,
}
