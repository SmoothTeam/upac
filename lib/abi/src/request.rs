// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::os::raw::c_void;

use upac_macro::{CNew, CValidate};

use crate::FileDiffKind;
use crate::error::ErrorKind;
use crate::hook::{CancelToken, HookMessageFn};
use crate::package::CPackageInfo;
use crate::types::{CSlice, CVec, check_size};

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CRequestBase {
    pub struct_size: usize,

    pub on_hook: Option<HookMessageFn>,
    pub hook_ctx: *mut c_void,

    pub cancel_token: *mut CancelToken,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CInstallRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub tmp_path: CSlice,

    pub subject: CSlice,
    #[optional]
    pub message: CSlice,

    pub packages: CVec<CSlice>,
    #[optional]
    pub boot_plugin: CSlice,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CUpdateRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub tmp_path: CSlice,

    pub subject: CSlice,
    #[optional]
    pub message: CSlice,

    pub packages: CVec<CSlice>,
    #[optional]
    pub boot_plugin: CSlice,
    pub allow_downgrade: bool,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CUninstallRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub tmp_path: CSlice,
    pub subject: CSlice,
    #[optional]
    pub message: CSlice,
    pub packages: CVec<CPackageInfo>,
    #[optional]
    pub boot_plugin: CSlice,
    pub purge: bool,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CRollbackRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub tmp_path: CSlice,
    pub config_digest: CSlice,
    #[optional]
    pub boot_plugin: CSlice,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CCommitRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub tmp_path: CSlice,
    pub subject: CSlice,
    #[optional]
    pub message: CSlice,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CFilesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub tmp_path: CSlice,
    pub subject: CSlice,
    #[optional]
    pub message: CSlice,
    pub files: CVec<CSlice>,
    pub file_kind: FileDiffKind,
    pub file_package: *const CPackageInfo,
    #[optional]
    pub boot_plugin: CSlice,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CGcRequest {
    pub struct_size: usize,
    pub base: CRequestBase,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CMimeSyncRequest {
    pub struct_size: usize,
    pub base: CRequestBase,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CListPackagesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CListConfigRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub prefix_digest: CSlice,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CListPrefixRequest {
    pub struct_size: usize,
    pub base: CRequestBase,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CListHistoryRequest {
    pub struct_size: usize,
    pub base: CRequestBase,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CDiffPrefixRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub from_prefix_digest: CSlice,
    #[optional]
    pub to_prefix_digest: CSlice,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CDiffConfigRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub from_config_digest: CSlice,
    #[optional]
    pub to_config_digest: CSlice,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CDiffPackagesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub from_prefix_digest: CSlice,
    #[optional]
    pub to_prefix_digest: CSlice,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CDiffRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub from_prefix_digest: CSlice,
    #[optional]
    pub to_prefix_digest: CSlice,
    #[optional]
    pub from_config_digest: CSlice,
    #[optional]
    pub to_config_digest: CSlice,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CSearchMetaRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub search: CSlice,
    pub is_regex: bool,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CSearchFilesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub search: CSlice,
    pub is_regex: bool,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CSearchInMetaRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub package: CPackageInfo,
    pub search: CSlice,
    pub is_regex: bool,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CSearchInPackageFilesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub package: CPackageInfo,
    pub search: CSlice,
    pub is_regex: bool,
}
