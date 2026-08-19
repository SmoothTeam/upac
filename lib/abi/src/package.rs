// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_macro::{CFree, CNew, CValidate};

use crate::error::ErrorKind;
use crate::memory::{free_cslice, free_cvec};
use crate::types::{CSlice, CVec, check_size};

#[repr(C)]
#[derive(CFree, CValidate)]
pub struct CVersion {
    pub struct_size: usize,

    pub epoch: u32,
    pub release: u32,
    #[non_empty]
    pub parts: CVec<u32>,
    #[optional]
    pub pre: CSlice,
}

#[repr(C)]
#[derive(CFree, CValidate)]
pub struct CPackageMeta {
    pub struct_size: usize,
    pub name: CSlice,
    pub version: CVersion,
    pub arch: CSlice,

    #[optional]
    pub arch_sub: CSlice,
    pub maintainer: CSlice,
    pub description: CSlice,
    #[optional]
    pub license: CSlice,
    #[optional]
    pub url: CSlice,
    pub sha256: [u8; 32],
    pub installed_size: u64,
}

#[repr(C)]
#[derive(CNew, CValidate)]
pub struct CPackageInfo {
    pub struct_size: usize,
    pub name: CSlice,
    pub arch: CSlice,
    #[optional]
    pub arch_sub: CSlice,
}
