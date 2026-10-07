// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{CFree, CNew, CValidate};

use super::CRequestBase;

use crate::package::CPackageInfo;
use crate::types::{CSlice, CVec};

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CInstallRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub boot_plugin: CSlice,

    pub tmp_path: CSlice,

    pub subject: CSlice,
    #[optional]
    pub message: CSlice,

    pub packages: CVec<CSlice>,

    pub allow_conflict_files: bool,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CUpdateRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub boot_plugin: CSlice,

    pub tmp_path: CSlice,

    pub subject: CSlice,
    #[optional]
    pub message: CSlice,

    pub packages: CVec<CSlice>,

    pub allow_downgrade: bool,
    pub allow_conflict_files: bool,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CUninstallRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub boot_plugin: CSlice,

    pub subject: CSlice,
    #[optional]
    pub message: CSlice,
    pub packages: CVec<CPackageInfo>,

    pub purge: bool,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CRollbackRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub boot_plugin: CSlice,

    pub config_digest: CSlice,

    pub discard_etc_changes: bool,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CCommitRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub subject: CSlice,
    #[optional]
    pub message: CSlice,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CFileTransfer {
    pub struct_size: usize,

    #[non_empty]
    pub source: CSlice,
    #[non_empty]
    pub target: CSlice,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CAttachRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub boot_plugin: CSlice,

    pub subject: CSlice,
    #[optional]
    pub message: CSlice,
    pub files: CVec<CFileTransfer>,
    pub file_package: CPackageInfo,

    pub scope: u8,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CDetachRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub boot_plugin: CSlice,

    pub subject: CSlice,
    #[optional]
    pub message: CSlice,
    pub files: CVec<CSlice>,
    pub file_package: CPackageInfo,

    pub scope: u8,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CGcRequest {
    pub struct_size: usize,
    pub base: CRequestBase,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CMimeSyncRequest {
    pub struct_size: usize,
    pub base: CRequestBase,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CPinRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[non_empty]
    pub prefix_digest: CSlice,
    pub pinned: bool,
}
