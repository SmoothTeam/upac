// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::os::raw::c_void;

use upac_abi::request::{
    CBootPluginDescriptorRequest, CBootPluginEntryRequest, CBootPluginInstallRequest, CDecodeRequest, CRequestBase,
    CSetupBootstrapDeployRequest, CSetupBootstrapImportRequest, CSetupBootstrapKernelRequest, CSetupFormatRequest,
    CSetupPartitionAddRequest, CSetupPartitionTableRequest, CreateFileFn, FinishFileFn,
};
use upac_abi::types::{COwned, CSlice};
use upac_abi::{CPackageChecksum, CPackageMeta, CProgressEvent, HookMessageFn};

use upac_macro::{CEnum, CTryToRust, RustToC};

use crate::booter::BootResourceKind;
use crate::error::ErrorKind;
use crate::package::{PackageChecksum, PackageMeta};
use crate::{CancelToken, ProgressEvent};

pub mod mutated;
pub mod unmutated;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, CEnum)]
pub enum FsKind {
    Ext4 = 0,
    Btrfs = 1,
    Xfs = 2,
    Vfat = 3,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, CEnum)]
pub enum PartitionKind {
    Esp = 0,
    Root = 1,
    Linux = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, CEnum)]
pub enum InitramfsGenerator {
    Dracut = 0,
    Mkinitcpio = 1,
}

pub trait Output<'data> {
    type Value;

    fn read_output(&'data self) -> Result<Self::Value, ErrorKind>;
    fn write_output(&mut self, value: Self::Value);
}

impl AsRef<str> for FsKind {
    fn as_ref(&self) -> &str {
        match self {
            FsKind::Ext4 => "ext4",
            FsKind::Btrfs => "btrfs",
            FsKind::Xfs => "xfs",
            FsKind::Vfat => "vfat",
        }
    }
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct RequestBase<'data> {
    pub on_hook: Option<HookMessageFn>,
    pub hook_ctx: *mut c_void,
    pub cancel_token: &'data CancelToken,
}

impl RequestBase<'_> {
    pub fn report_progress(&self, event: &ProgressEvent<'_>) {
        let Some(on_hook) = self.on_hook else {
            return;
        };

        let c_event = CProgressEvent::from(event.clone());
        unsafe { on_hook(&c_event, self.hook_ctx) };
        unsafe { c_event.free() };
    }
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SetupFormatRequest<'data> {
    pub base: RequestBase<'data>,

    pub device_path: &'data str,
    pub label: Option<&'data str>,
    pub fs_kind: FsKind,
    pub require_esp: bool,
    pub force_wipe: bool,
    pub btrfs_node_size: u32,
    pub btrfs_sector_size: u32,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SetupPartitionTableRequest<'data> {
    pub base: RequestBase<'data>,

    pub device_path: &'data str,
    pub force_wipe: bool,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SetupPartitionAddRequest<'data> {
    pub base: RequestBase<'data>,

    pub device_path: &'data str,
    pub label: &'data str,
    pub size_mib: u64,
    pub kind: PartitionKind,

    #[out]
    pub partition_label: String,
    #[out]
    pub partition_device_path: String,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SetupBootstrapImportRequest<'data> {
    pub base: RequestBase<'data>,

    pub disk: Option<&'data str>,
    pub deploy_device: Option<&'data str>,

    pub mount_point: Option<&'data str>,
    pub tmp_path: Option<&'data str>,
    pub source: &'data str,

    #[out]
    pub image_digest: String,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SetupBootstrapKernelRequest<'data> {
    pub base: RequestBase<'data>,

    pub disk: Option<&'data str>,
    pub deploy_device: Option<&'data str>,

    pub mount_point: Option<&'data str>,
    pub tmp_path: Option<&'data str>,
    pub digest: &'data str,
    pub initramfs_generator: InitramfsGenerator,

    #[out]
    pub image_digest: String,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SetupBootstrapDeployRequest<'data> {
    pub base: RequestBase<'data>,

    pub disk: Option<&'data str>,
    pub esp_device: Option<&'data str>,
    pub deploy_device: Option<&'data str>,

    pub mount_point: Option<&'data str>,
    pub digest: &'data str,
    pub boot_plugin: &'data str,
    pub pinned: bool,
}

#[derive(Debug, Clone, RustToC)]
pub struct BootPluginDescriptorRequest {
    #[out]
    pub abi_version: u32,
    #[out]
    pub kind: BootResourceKind,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct BootPluginInstallRequest {
    pub esp_mount_point: String,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct BootPluginEntryRequest {
    pub esp_mount_point: String,

    pub entry_id: String,
    pub entry_path: String,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct DecodeRequest<'data> {
    pub package_path: &'data str,
    pub checksum: PackageChecksum,
    pub create_file: CreateFileFn,
    pub finish_file: FinishFileFn,
    pub store_ctx: *mut c_void,
    pub cancel_token: &'data CancelToken,

    #[out]
    pub meta: PackageMeta,
    #[out]
    pub dumpfile: String,
}
