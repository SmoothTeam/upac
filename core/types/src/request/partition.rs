// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::FsKind;
use upac_abi::error::ErrorKind;
use upac_abi::request::CRequestBase;
use upac_abi::request::partition::{CPartitionMount, CPartitionSpec, CSetupPartitionRequest};
use upac_abi::types::{COwned, CSlice, CVec};

use upac_macro::{CTryToRust, RustToC};

use super::RequestBase;

#[derive(Debug, Clone, RustToC)]
pub struct SetupPartitionRequest {
    pub base: RequestBase,

    pub device_path: String,
    pub esp_size_mib: u64,
    pub deploy_size_mib: u64,
    pub extra_partitions: Vec<PartitionSpec>,
    pub force_wipe: bool,
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
