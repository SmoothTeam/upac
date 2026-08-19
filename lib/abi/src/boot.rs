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

pub trait Booter: Sized {
    type Error;

    fn new() -> Result<Self, Self::Error>;
    fn probes() -> bool;
    fn set_one_shot(&mut self, entry_name: &str) -> Result<(), Self::Error>;
    fn confirm_boot(&mut self, entry_name: &str) -> Result<(), Self::Error>;
}

#[repr(C)]
#[derive(CNew)]
pub struct CBootPluginRequest {
    pub struct_size: usize,

    pub entry_name: CSlice,
}
