// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::os::raw::c_void;
use std::sync::atomic::AtomicU8;

use upac_macro::{CFree, CNew, CValidate};

use crate::types::CSlice;
use crate::{CPackageChecksum, HookMessageFn};

pub mod mutated;
pub mod unmutated;

pub type CreateFileFn = unsafe extern "C" fn(store_ctx: *mut c_void, fd_out: *mut i32) -> i32;

pub type FinishFileFn =
    unsafe extern "C" fn(store_ctx: *mut c_void, fd: i32, size: u64, object_id_out: *mut [u8; 32]) -> i32;

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CRequestBase {
    pub struct_size: usize,

    pub on_hook: Option<HookMessageFn>,
    pub hook_ctx: *mut c_void,

    pub cancel_token: *const AtomicU8,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CSetupFormatRequest {
    pub struct_size: usize,

    pub base: CRequestBase,

    pub device_path: CSlice,
    #[optional]
    pub label: CSlice,
    pub fs_kind: u8,
    pub require_esp: bool,
    pub force_wipe: bool,
    pub btrfs_node_size: u32,
    pub btrfs_sector_size: u32,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CSetupPartitionTableRequest {
    pub struct_size: usize,

    pub base: CRequestBase,

    pub device_path: CSlice,
    pub force_wipe: bool,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CSetupPartitionAddRequest {
    pub struct_size: usize,

    pub base: CRequestBase,

    pub device_path: CSlice,
    pub label: CSlice,
    pub size_mib: u64,
    pub kind: u8,
}

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
    pub digest: CSlice,
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
    pub digest: CSlice,
    pub boot_plugin: CSlice,
    pub pinned: bool,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CBootPluginInstallRequest {
    pub struct_size: usize,

    pub esp_mount_point: CSlice,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CBootPluginEntryRequest {
    pub struct_size: usize,

    pub esp_mount_point: CSlice,
    pub entry_id: CSlice,
    pub entry_path: CSlice,
}

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
