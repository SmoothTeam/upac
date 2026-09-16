// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::mem::size_of;
use std::os::raw::c_void;

use upac_abi::HookMessageFn;
use upac_abi::error::ErrorKind;
use upac_abi::hook::CancelToken;
use upac_abi::package::CPackageInfo;
use upac_abi::request::{
    CBootPluginConfirmSuccsesBootRequest, CBootPluginInstallRequest, CBootPluginSetOneShotRequest, CBtrfsOptions,
    CCommitRequest, CDecodeRequest, CDiffConfigRequest, CDiffPackagesRequest, CDiffPrefixRequest, CDiffRequest,
    CFilesRequest, CGcRequest, CGptLayout, CInstallRequest, CListConfigRequest, CListHistoryRequest,
    CListPackagesRequest, CListPrefixRequest, CMimeSyncRequest, CPartitionMount, CPartitionSpec, CPinRequest,
    CRequestBase, CRollbackRequest, CSearchFilesRequest, CSearchInMetaRequest, CSearchInPackageFilesRequest,
    CSearchMetaRequest, CSetupExistingRequest, CSetupWholeDiskRequest, CUninstallRequest, CUpdateRequest,
};
use upac_abi::types::{COwned, CSlice, CVec};
use upac_abi::{DiffFileSource, FileDiffKind, FsKind};

use upac_macro::{CTryToRust, RustToC};

use super::package::PackageInfo;

#[derive(Debug, Clone, RustToC)]
pub struct RequestBase {
    pub on_hook: Option<HookMessageFn>,
    pub hook_ctx: *mut c_void,
    pub cancel_token: *mut CancelToken,
}

#[derive(Debug, Clone, RustToC)]
pub struct InstallRequest {
    pub base: RequestBase,

    pub tmp_path: String,

    pub subject: String,
    pub message: Option<String>,
    pub packages: Vec<String>,

    pub boot_plugin: String,

    pub allow_conflict_files: bool,
}

#[derive(Debug, Clone, RustToC)]
pub struct UpdateRequest {
    pub base: RequestBase,
    pub tmp_path: String,

    pub subject: String,
    pub message: Option<String>,

    pub packages: Vec<String>,

    pub boot_plugin: String,

    pub allow_downgrade: bool,
    pub allow_conflict_files: bool,
}

#[derive(Debug, Clone, RustToC)]
pub struct UninstallRequest {
    pub base: RequestBase,

    pub tmp_path: String,

    pub subject: String,
    pub message: Option<String>,

    pub packages: Vec<PackageInfo>,

    pub boot_plugin: String,

    pub purge: bool,
}

#[derive(Debug, Clone, RustToC)]
pub struct RollbackRequest {
    pub base: RequestBase,
    pub tmp_path: String,
    pub config_digest: String,
    pub boot_plugin: String,
}

#[derive(Debug, Clone, RustToC)]
pub struct CommitRequest {
    pub base: RequestBase,
    pub tmp_path: String,
    pub subject: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, RustToC)]
pub struct FilesRequest {
    pub base: RequestBase,

    pub tmp_path: String,

    pub subject: String,
    pub message: Option<String>,

    pub files: Vec<String>,
    pub file_kind: FileDiffKind,
    pub file_package: *const CPackageInfo,

    pub boot_plugin: String,

    pub scope: DiffFileSource,
}

#[derive(Debug, Clone, RustToC)]
pub struct GcRequest {
    pub base: RequestBase,
}

#[derive(Debug, Clone, RustToC)]
pub struct MimeSyncRequest {
    pub base: RequestBase,
}

#[derive(Debug, Clone, RustToC)]
pub struct PinRequest {
    pub base: RequestBase,
    pub prefix_digest: String,
    pub pinned: bool,
}

#[derive(Debug, Clone, RustToC)]
pub struct ListPackagesRequest {
    pub base: RequestBase,
}

#[derive(Debug, Clone, RustToC)]
pub struct ListConfigRequest {
    pub base: RequestBase,
    pub prefix_digest: Option<String>,
}

#[derive(Debug, Clone, RustToC)]
pub struct ListPrefixRequest {
    pub base: RequestBase,
}

#[derive(Debug, Clone, RustToC)]
pub struct ListHistoryRequest {
    pub base: RequestBase,
}

#[derive(Debug, Clone, RustToC)]
pub struct DiffPrefixRequest {
    pub base: RequestBase,
    pub from_prefix_digest: Option<String>,
    pub to_prefix_digest: Option<String>,
}

#[derive(Debug, Clone, RustToC)]
pub struct DiffConfigRequest {
    pub base: RequestBase,
    pub from_config_digest: Option<String>,
    pub to_config_digest: Option<String>,
}

#[derive(Debug, Clone, RustToC)]
pub struct DiffPackagesRequest {
    pub base: RequestBase,
    pub from_prefix_digest: Option<String>,
    pub to_prefix_digest: Option<String>,
}

#[derive(Debug, Clone, RustToC)]
pub struct DiffRequest {
    pub base: RequestBase,
    pub from_prefix_digest: Option<String>,
    pub to_prefix_digest: Option<String>,
    pub from_config_digest: Option<String>,
    pub to_config_digest: Option<String>,
}

#[derive(Debug, Clone, RustToC)]
pub struct SearchMetaRequest {
    pub base: RequestBase,
    pub search: String,
    pub is_regex: bool,
}

#[derive(Debug, Clone, RustToC)]
pub struct SearchFilesRequest {
    pub base: RequestBase,
    pub search: String,
    pub is_regex: bool,
}

#[derive(Debug, Clone, RustToC)]
pub struct SearchInMetaRequest {
    pub base: RequestBase,
    pub package: PackageInfo,
    pub search: String,
    pub is_regex: bool,
}

#[derive(Debug, Clone, RustToC)]
pub struct SearchInPackageFilesRequest {
    pub base: RequestBase,
    pub package: PackageInfo,
    pub search: String,
    pub is_regex: bool,
}

#[derive(Debug, Clone, RustToC)]
pub struct DecodeRequest {
    pub package_path: String,
    pub output_dir: String,
    pub checksum: [u8; 32],
    pub cancel_token: *mut CancelToken,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct BootPluginSetOneShotRequest {
    pub entry_name: String,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct BootPluginConfirmSuccsesBootRequest {
    pub entry_name: String,

    pub esp_mount_point: String,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct BootPluginInstallRequest {
    pub esp_mount_point: String,
    pub esp_partition_number: u32,
    pub esp_starting_lba: u64,
    pub esp_ending_lba: u64,
    pub esp_unique_partition_guid: [u8; 16],

    pub to_slot: String,
    pub from_slot: String,
}

#[derive(Debug, Clone, CTryToRust, RustToC)]
pub struct PartitionMount {
    pub mount_path: String,
    pub device_path: String,
    pub fs_kind: FsKind,
}

#[derive(Debug, Clone, CTryToRust, RustToC)]
pub struct PartitionSpec {
    pub mount_path: String,
    pub size_mib: u64,
    pub fs_kind: FsKind,
}

#[derive(Debug, Clone, CTryToRust, RustToC)]
pub struct GptLayout {
    pub esp_size_mib: u64,
    pub deploy_fs: FsKind,
    pub deploy_size_mib: u64,
    pub extra_partitions: Vec<PartitionSpec>,
    pub force_wipe: bool,
}

#[derive(Debug, Clone, CTryToRust, RustToC)]
pub struct BtrfsOptions {
    pub node_size: u32,
    pub sector_size: u32,
}

#[derive(Debug, Clone, RustToC)]
pub struct SetupExistingRequest {
    pub base: RequestBase,

    pub esp_device: String,
    pub deploy_device: String,
    pub deploy_fs: FsKind,
    pub extra_mounts: Vec<PartitionMount>,

    pub mount_point: Option<String>,
    pub source: String,
    pub empty_config: bool,
    pub pinned: bool,
    pub boot_plugin: String,
}

#[derive(Debug, Clone, RustToC)]
pub struct SetupWholeDiskRequest {
    pub base: RequestBase,

    pub device_path: String,
    pub gpt: GptLayout,
    pub btrfs: BtrfsOptions,

    pub mount_point: Option<String>,
    pub source: String,
    pub empty_config: bool,
    pub pinned: bool,
    pub boot_plugin: String,
}
