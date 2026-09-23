// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::ErrorKind;
use upac_abi::package::CPackageInfo;
use upac_abi::request::CRequestBase;
use upac_abi::request::unmutated::{
    CDiffConfigRequest, CDiffPackagesRequest, CDiffPrefixRequest, CDiffRequest, CListConfigRequest,
    CListHistoryRequest, CListPackagesRequest, CListPrefixRequest, CSearchFilesRequest, CSearchInMetaRequest,
    CSearchInPackageFilesRequest, CSearchMetaRequest,
};
use upac_abi::types::{COwned, CSlice};

use upac_macro::{CTryToRust, RustToC};

use super::RequestBase;

use crate::package::PackageInfo;

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct ListPackagesRequest {
    pub base: RequestBase,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct ListConfigRequest<'data> {
    pub base: RequestBase,
    pub prefix_digest: Option<&'data str>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct ListPrefixRequest {
    pub base: RequestBase,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct ListHistoryRequest {
    pub base: RequestBase,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct DiffPrefixRequest<'data> {
    pub base: RequestBase,
    pub from_prefix_digest: Option<&'data str>,
    pub to_prefix_digest: Option<&'data str>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct DiffConfigRequest<'data> {
    pub base: RequestBase,
    pub from_config_digest: Option<&'data str>,
    pub to_config_digest: Option<&'data str>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct DiffPackagesRequest<'data> {
    pub base: RequestBase,
    pub from_prefix_digest: Option<&'data str>,
    pub to_prefix_digest: Option<&'data str>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct DiffRequest<'data> {
    pub base: RequestBase,
    pub from_prefix_digest: Option<&'data str>,
    pub to_prefix_digest: Option<&'data str>,
    pub from_config_digest: Option<&'data str>,
    pub to_config_digest: Option<&'data str>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SearchMetaRequest<'data> {
    pub base: RequestBase,
    pub search: &'data str,
    pub is_regex: bool,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SearchFilesRequest<'data> {
    pub base: RequestBase,
    pub search: &'data str,
    pub is_regex: bool,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SearchInMetaRequest<'data> {
    pub base: RequestBase,
    pub package: PackageInfo,
    pub search: &'data str,
    pub is_regex: bool,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SearchInPackageFilesRequest<'data> {
    pub base: RequestBase,
    pub package: PackageInfo,
    pub search: &'data str,
    pub is_regex: bool,
}
