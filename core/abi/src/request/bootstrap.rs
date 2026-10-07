// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{CFree, CNew, CValidate};

use super::CRequestBase;

use crate::types::CSlice;

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CSetupBootstrapImportRequest {
    pub struct_size: usize,

    pub base: CRequestBase,

    #[optional]
    pub disk: CSlice,
    #[optional]
    pub deploy_device: CSlice,

    #[optional]
    pub mount_point: CSlice,
    #[optional]
    pub tmp_path: CSlice,
    pub source: CSlice,
    pub empty_config: bool,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CSetupBootstrapKernelRequest {
    pub struct_size: usize,

    pub base: CRequestBase,

    #[optional]
    pub disk: CSlice,
    #[optional]
    pub deploy_device: CSlice,

    #[optional]
    pub mount_point: CSlice,
    #[optional]
    pub tmp_path: CSlice,
    pub prefix_digest: CSlice,
    pub boot_plugin: CSlice,
    pub initramfs_generator: u8,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CSetupBootstrapDeployRequest {
    pub struct_size: usize,

    pub base: CRequestBase,

    #[optional]
    pub disk: CSlice,
    #[optional]
    pub esp_device: CSlice,
    #[optional]
    pub deploy_device: CSlice,

    #[optional]
    pub mount_point: CSlice,
    pub prefix_digest: CSlice,
    pub config_digest: CSlice,
    pub boot_plugin: CSlice,
    pub pinned: bool,
}
