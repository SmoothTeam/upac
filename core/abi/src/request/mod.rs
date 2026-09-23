// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::os::raw::c_void;

use upac_macro::{CFree, CNew, CValidate};

use super::HookMessageFn;

use crate::hook::CancelToken;
use crate::types::{CSlice, CVec};
use crate::{FsKind, InitramfsGenerator};

use self::partition::CPartitionMount;

pub mod booter;
pub mod decoder;
pub mod format;
pub mod mutated;
pub mod partition;
pub mod unmutated;

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CRequestBase {
    pub struct_size: usize,

    pub on_hook: Option<HookMessageFn>,
    pub hook_ctx: *mut c_void,

    pub cancel_token: *mut CancelToken,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CSetupExistingRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub esp_device: CSlice,
    pub deploy_device: CSlice,
    pub deploy_fs: FsKind,
    pub extra_mounts: CVec<CPartitionMount>,

    #[optional]
    pub mount_point: CSlice,
    pub source: CSlice,
    pub empty_config: bool,
    pub pinned: bool,
    pub boot_plugin: CSlice,
    pub initramfs_generator: InitramfsGenerator,
}
