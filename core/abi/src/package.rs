// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{CFree, CNew, CValidate};

use crate::CONSTRAINT_ANY;
use crate::types::CSlice;

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CVersion {
    pub struct_size: usize,

    pub epoch: u32,
    #[non_empty]
    pub raw: CSlice,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
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
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CPackageInfo {
    pub struct_size: usize,
    pub name: CSlice,
    pub arch: CSlice,
    #[optional]
    pub arch_sub: CSlice,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CPackageDependency {
    pub struct_size: usize,

    pub name: CSlice,
    #[bitflags(CONSTRAINT_ANY)]
    pub constraint: u8,
    #[skip_if(constraint == CONSTRAINT_ANY)]
    pub version: CVersion,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CPackageTrigger {
    pub struct_size: usize,

    pub position: u8,
    #[non_empty]
    pub name: CSlice,
}
