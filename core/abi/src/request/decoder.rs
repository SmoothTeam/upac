// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::os::raw::c_void;
use std::sync::atomic::AtomicU8;

use upac_macro::{CFree, CNew, CValidate};

use crate::package::CPackageChecksum;
use crate::plugin::{CreateFileFn, FinishFileFn};
use crate::types::CSlice;

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CDecodeRequest {
    pub struct_size: usize,

    pub package_path: CSlice,

    pub checksum: CPackageChecksum,

    pub create_file: CreateFileFn,
    pub finish_file: FinishFileFn,
    pub store_ctx: *mut c_void,

    pub cancel_token: *const AtomicU8,
}
