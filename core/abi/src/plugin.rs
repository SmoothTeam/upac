// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::CEnum;

use crate::request::booter::{
    CBootPluginConfirmSuccessBootRequest, CBootPluginInstallRequest, CBootPluginSetOneShotRequest,
};
use crate::request::decoder::CDecodeRequest;
use crate::response::decoder::CDecodeResponse;

pub type BootPluginAbiVersionFn = unsafe extern "C" fn() -> u32;

pub type DecodePluginAbiVersionFn = unsafe extern "C" fn() -> u32;

pub type SetOneShotFn = unsafe extern "C" fn(request: *const CBootPluginSetOneShotRequest) -> i32;

pub type ConfirmBootFn = unsafe extern "C" fn(request: *const CBootPluginConfirmSuccessBootRequest) -> i32;

pub type InstallFn = unsafe extern "C" fn(request: *const CBootPluginInstallRequest) -> i32;

pub type BootResourceKindFn = unsafe extern "C" fn() -> u8;

pub type DecodeFn = unsafe extern "C" fn(request: *const CDecodeRequest, response_out: *mut CDecodeResponse) -> i32;

pub type FreeDecodeResponseFn = unsafe extern "C" fn(response: *mut CDecodeResponse);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, CEnum)]
pub enum BootResourceKind {
    Bls = 0,
    Uki = 1,
}
