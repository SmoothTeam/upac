// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_macro::CNew;

use crate::types::CSlice;

pub type AbiVersionFn = unsafe extern "C" fn() -> u32;

pub type ProbeFn = unsafe extern "C" fn() -> i32;

pub type SetOneShotFn = unsafe extern "C" fn(request: *const CBootPluginRequest) -> i32;

pub type ConfirmBootFn = unsafe extern "C" fn(request: *const CBootPluginRequest) -> i32;

#[repr(C)]
#[derive(CNew)]
pub struct CBootPluginRequest {
    pub struct_size: usize,

    pub entry_name: CSlice,
}
