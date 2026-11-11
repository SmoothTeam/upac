// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::request::mutated::{
    CAttachRequest, CDetachRequest, CFileTransfer, CGcRequest, CInstallRequest, CMimeSyncRequest, CPinRequest,
    CRollbackRequest, CUninstallRequest, CUpdateRequest,
};

use upac_abi::CPackageId;
use upac_abi::request::CRequestBase;
use upac_abi::types::{COwned, CSlice, CVec};

use upac_macro::{CTryToRust, RustToC};

use super::RequestBase;

use crate::error::ErrorKind;
use crate::package::PackageId;

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct InstallRequest<'data> {
    pub base: RequestBase<'data>,

    pub subject: &'data str,
    pub message: Option<&'data str>,

    pub packages: Vec<&'data str>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct UpdateRequest<'data> {
    pub base: RequestBase<'data>,

    pub subject: &'data str,
    pub message: Option<&'data str>,

    pub packages: Vec<&'data str>,

    pub allow_downgrade: bool,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct UninstallRequest<'data> {
    pub base: RequestBase<'data>,

    pub subject: &'data str,
    pub message: Option<&'data str>,

    pub packages: Vec<PackageId>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct RollbackRequest<'data> {
    pub base: RequestBase<'data>,
    pub digest: &'data str,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct FileTransfer<'data> {
    pub source: &'data str,
    pub target: &'data str,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct AttachRequest<'data> {
    pub base: RequestBase<'data>,

    pub subject: &'data str,
    pub message: Option<&'data str>,

    pub files: Vec<FileTransfer<'data>>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct DetachRequest<'data> {
    pub base: RequestBase<'data>,

    pub subject: &'data str,
    pub message: Option<&'data str>,

    pub files: Vec<&'data str>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct GcRequest<'data> {
    pub base: RequestBase<'data>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct MimeSyncRequest<'data> {
    pub base: RequestBase<'data>,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct PinRequest<'data> {
    pub base: RequestBase<'data>,
    pub digest: &'data str,
    pub pinned: bool,
}
