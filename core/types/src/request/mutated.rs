// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::request::mutated::{
    CCommitRequest, CFilesRequest, CGcRequest, CInstallRequest, CMimeSyncRequest, CPinRequest, CRollbackRequest,
    CUninstallRequest, CUpdateRequest,
};

use upac_abi::package::CPackageInfo;
use upac_abi::request::CRequestBase;
use upac_abi::types::{COwned, CSlice, CVec};
use upac_abi::{DiffFileSource, FileDiffKind};

use upac_macro::RustToC;

use super::RequestBase;

use crate::package::PackageInfo;

#[derive(Debug, Clone, RustToC)]
pub struct InstallRequest {
    pub base: RequestBase,

    pub tmp_path: String,

    pub subject: String,
    pub message: Option<String>,
    pub packages: Vec<String>,

    pub boot_plugin: String,

    pub allow_conflict_files: bool,
}

#[derive(Debug, Clone, RustToC)]
pub struct UpdateRequest {
    pub base: RequestBase,
    pub tmp_path: String,

    pub subject: String,
    pub message: Option<String>,

    pub packages: Vec<String>,

    pub boot_plugin: String,

    pub allow_downgrade: bool,
    pub allow_conflict_files: bool,
}

#[derive(Debug, Clone, RustToC)]
pub struct UninstallRequest {
    pub base: RequestBase,

    pub tmp_path: String,

    pub subject: String,
    pub message: Option<String>,

    pub packages: Vec<PackageInfo>,

    pub boot_plugin: String,

    pub purge: bool,
}

#[derive(Debug, Clone, RustToC)]
pub struct RollbackRequest {
    pub base: RequestBase,
    pub tmp_path: String,
    pub config_digest: String,
    pub boot_plugin: String,
}

#[derive(Debug, Clone, RustToC)]
pub struct CommitRequest {
    pub base: RequestBase,
    pub tmp_path: String,
    pub subject: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, RustToC)]
pub struct FilesRequest {
    pub base: RequestBase,

    pub tmp_path: String,

    pub subject: String,
    pub message: Option<String>,

    pub files: Vec<String>,
    pub file_kind: FileDiffKind,
    pub file_package: *const CPackageInfo,

    pub boot_plugin: String,

    pub scope: DiffFileSource,
}

#[derive(Debug, Clone, RustToC)]
pub struct GcRequest {
    pub base: RequestBase,
}

#[derive(Debug, Clone, RustToC)]
pub struct MimeSyncRequest {
    pub base: RequestBase,
}

#[derive(Debug, Clone, RustToC)]
pub struct PinRequest {
    pub base: RequestBase,
    pub prefix_digest: String,
    pub pinned: bool,
}
