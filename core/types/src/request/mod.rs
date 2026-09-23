// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::os::raw::c_void;

use upac_abi::HookMessageFn;
use upac_abi::error::ErrorKind;
use upac_abi::hook::CancelToken;
use upac_abi::request::CRequestBase;
use upac_abi::request::CSetupExistingRequest;
use upac_abi::request::partition::CPartitionMount;
use upac_abi::types::{COwned, CSlice, CVec};
use upac_abi::{FsKind, InitramfsGenerator};

use upac_macro::{CTryToRust, RustToC};

use self::partition::PartitionMount;

pub mod booter;
pub mod decoder;
pub mod format;
pub mod mutated;
pub mod partition;
pub mod unmutated;

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct RequestBase {
    pub on_hook: Option<HookMessageFn>,
    pub hook_ctx: *mut c_void,
    pub cancel_token: *mut CancelToken,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SetupExistingRequest<'data> {
    pub base: RequestBase,

    pub esp_device: &'data str,
    pub deploy_device: &'data str,
    pub deploy_fs: FsKind,
    pub extra_mounts: Vec<PartitionMount>,

    pub mount_point: Option<&'data str>,
    pub source: &'data str,
    pub empty_config: bool,
    pub pinned: bool,

    pub boot_plugin: &'data str,
    pub initramfs_generator: InitramfsGenerator,
}
