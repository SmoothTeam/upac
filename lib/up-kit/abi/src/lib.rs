// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

extern crate self as upac_abi;

use std::ffi::c_void;

use upac_macro::{CFree, CNew, CValidate};

use crate::types::CSlice;

pub mod error;
pub mod memory;
pub mod request;
pub mod response;
pub mod types;

pub const LIB_ABI_VERSION: u32 = 3;
pub const BOOT_ABI_VERSION: u32 = 3;
pub const DECODER_ABI_VERSION: u32 = 3;
pub const SETUP_ABI_VERSION: u32 = 3;

pub const OWNER_XATTR: &str = "user.upac.owner";
pub const OWNER_SEPARATOR: &str = ":";

pub const CONSTRAINT_LESS: u8 = 0b001;
pub const CONSTRAINT_EQUAL: u8 = 0b010;
pub const CONSTRAINT_GREATER: u8 = 0b100;
pub const CONSTRAINT_ANY: u8 = CONSTRAINT_LESS | CONSTRAINT_EQUAL | CONSTRAINT_GREATER;

pub type CPackageChecksum = [u8; 32];

pub type HookMessageFn = unsafe extern "C" fn(event: *const CProgressEvent, ctx: *mut c_void);

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
    pub checksum: CPackageChecksum,
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

#[repr(C)]
#[derive(CFree)]
pub struct CProgressEvent {
    pub struct_size: usize,
    pub stage: u32,
    pub subject: CSlice,
    pub current: u64,
    pub total: u64,
}
