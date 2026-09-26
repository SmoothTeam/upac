// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::request::mutated::{
    CCommitRequest, CFilesRequest, CGcRequest, CInstallRequest, CMimeSyncRequest, CPinRequest, CRollbackRequest,
    CUninstallRequest, CUpdateRequest,
};

use upac_abi::error::ErrorKind;
use upac_abi::package::CPackageInfo;
use upac_abi::request::CRequestBase;
use upac_abi::response::entry::{DiffFileSource, FileDiffKind};
use upac_abi::types::{COwned, CSlice, CVec};

use upac_macro::{CTryToRust, RustToC};

use super::RequestBase;

use crate::package::PackageInfo;

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct InstallRequest<'data> {
    pub base: RequestBase,

    pub tmp_path: &'data str,

    pub subject: &'data str,
    pub message: Option<&'data str>,

    pub packages: Vec<&'data str>,

    pub boot_plugin: &'data str,

    pub allow_conflict_files: bool,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct UpdateRequest<'data> {
    pub base: RequestBase,

    pub tmp_path: &'data str,

    pub subject: &'data str,
    pub message: Option<&'data str>,

    pub packages: Vec<&'data str>,

    pub boot_plugin: &'data str,

    pub allow_downgrade: bool,
    pub allow_conflict_files: bool,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct UninstallRequest<'data> {
    pub base: RequestBase,

    pub tmp_path: &'data str,

    pub subject: &'data str,
    pub message: Option<&'data str>,

    pub packages: Vec<PackageInfo>,

    pub boot_plugin: &'data str,

    pub purge: bool,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct RollbackRequest<'data> {
    pub base: RequestBase,
    pub tmp_path: &'data str,
    pub config_digest: &'data str,
    pub boot_plugin: &'data str,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct CommitRequest<'data> {
    pub base: RequestBase,
    pub tmp_path: &'data str,
    pub subject: &'data str,
    pub message: Option<&'data str>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct FilesRequest<'data> {
    pub base: RequestBase,

    pub tmp_path: &'data str,

    pub subject: &'data str,
    pub message: Option<&'data str>,

    pub files: Vec<&'data str>,
    pub file_kind: FileDiffKind,
    pub file_package: *const CPackageInfo,

    pub boot_plugin: &'data str,

    pub scope: DiffFileSource,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct GcRequest {
    pub base: RequestBase,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct MimeSyncRequest {
    pub base: RequestBase,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct PinRequest<'data> {
    pub base: RequestBase,
    pub prefix_digest: &'data str,
    pub pinned: bool,
}
