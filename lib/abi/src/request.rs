// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::os::raw::c_void;

use upac_macro::CValidate;

use crate::FileDiffKind;
use crate::error::ErrorKind;
use crate::hook::{CancelToken, HookMessageFn};
use crate::package::{CPackageInfo, CUnpackedPackage};
use crate::types::{CSlice, CVec, check_size};

#[repr(C)]
#[derive(CValidate)]
pub struct CRequestBase {
    pub struct_size: usize,

    pub on_hook: Option<HookMessageFn>,
    pub hook_ctx: *mut c_void,

    pub cancel_token: *mut CancelToken,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CInstallRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub tmp_path: CSlice,

    pub subject: CSlice,
    #[optional]
    pub message: CSlice,

    pub packages: CVec<CUnpackedPackage>,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CUpdateRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub tmp_path: CSlice,

    pub subject: CSlice,
    #[optional]
    pub message: CSlice,

    pub packages: CVec<CUnpackedPackage>,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CUninstallRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub tmp_path: CSlice,
    pub subject: CSlice,
    #[optional]
    pub message: CSlice,
    pub packages: CVec<CPackageInfo>,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CRollbackRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub tmp_path: CSlice,
    pub config_digest: CSlice,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CCommitRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub tmp_path: CSlice,
    pub subject: CSlice,
    #[optional]
    pub message: CSlice,
}

#[repr(C)]
#[derive(CValidate)]
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
}

#[repr(C)]
#[derive(CValidate)]
pub struct CListPackagesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CListCommitRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub prefix_digest: CSlice,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CListPrefixRequest {
    pub struct_size: usize,
    pub base: CRequestBase,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CListHistoryRequest {
    pub struct_size: usize,
    pub base: CRequestBase,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CDiffFilesPrefixRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub from_prefix_digest: CSlice,
    #[optional]
    pub to_prefix_digest: CSlice,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CDiffFilesConfigRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub from_config_digest: CSlice,
    #[optional]
    pub to_config_digest: CSlice,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CDiffPackagesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub from_prefix_digest: CSlice,
    #[optional]
    pub to_prefix_digest: CSlice,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CDiffRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub from_prefix_digest: CSlice,
    #[optional]
    pub to_prefix_digest: CSlice,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CSearchMetaRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub search: CSlice,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CSearchFilesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub search: CSlice,
}
