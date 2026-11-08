// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::os::raw::c_void;

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

pub type CreateFileFn = unsafe extern "C" fn(store_ctx: *mut c_void, fd_out: *mut i32) -> i32;

pub type FinishFileFn =
    unsafe extern "C" fn(store_ctx: *mut c_void, fd: i32, size: u64, object_id_out: *mut [u8; 32]) -> i32;
