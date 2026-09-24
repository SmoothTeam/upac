// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{CFree, CNew, CValidate};

use super::CRequestBase;

use crate::types::CSlice;
use crate::{FsKind, InitramfsGenerator};

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CSetupBootstrapRequest {
    pub struct_size: usize,

    pub base: CRequestBase,

    #[optional]
    pub disk: CSlice,
    #[optional]
    pub esp_device: CSlice,
    #[optional]
    pub deploy_device: CSlice,
    pub deploy_fs: FsKind,

    #[optional]
    pub mount_point: CSlice,
    pub source: CSlice,
    pub empty_config: bool,
    pub pinned: bool,
    pub boot_plugin: CSlice,
    pub initramfs_generator: InitramfsGenerator,
}
