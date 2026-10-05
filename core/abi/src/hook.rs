// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::c_void;

use upac_macro::CFree;

use crate::types::CSlice;

pub type HookMessageFn = unsafe extern "C" fn(event: *const CProgressEvent, ctx: *mut c_void);

#[repr(C)]
#[derive(CFree)]
pub struct CProgressEvent {
    pub struct_size: usize,
    pub stage: u32,
    pub subject: CSlice,
    pub current: u64,
    pub total: u64,
}
