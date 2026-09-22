// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{CFree, CNew, CValidate};

use super::CRequestBase;

use crate::error::ErrorKind;
use crate::memory::free_cslice;
use crate::package::CPackageInfo;
use crate::types::{CSlice, check_size};

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CListPackagesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CListConfigRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub prefix_digest: CSlice,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CListPrefixRequest {
    pub struct_size: usize,
    pub base: CRequestBase,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CListHistoryRequest {
    pub struct_size: usize,
    pub base: CRequestBase,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CDiffPrefixRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub from_prefix_digest: CSlice,
    #[optional]
    pub to_prefix_digest: CSlice,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CDiffConfigRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub from_config_digest: CSlice,
    #[optional]
    pub to_config_digest: CSlice,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CDiffPackagesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub from_prefix_digest: CSlice,
    #[optional]
    pub to_prefix_digest: CSlice,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
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
#[derive(CFree, CNew, CValidate)]
pub struct CSearchMetaRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub search: CSlice,
    pub is_regex: bool,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CSearchFilesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub search: CSlice,
    pub is_regex: bool,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CSearchInMetaRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub package: CPackageInfo,
    pub search: CSlice,
    pub is_regex: bool,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CSearchInPackageFilesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub package: CPackageInfo,
    pub search: CSlice,
    pub is_regex: bool,
}
