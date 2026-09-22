// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::FsKind;
use upac_abi::error::ErrorKind;
use upac_abi::request::CRequestBase;
use upac_abi::request::format::{CFormatPartitionSpec, CSetupFormatRequest};
use upac_abi::types::{COwned, CSlice, CVec};

use upac_macro::{CTryToRust, RustToC};

use super::RequestBase;

#[derive(Debug, Clone, RustToC)]
pub struct SetupFormatRequest {
    pub base: RequestBase,

    pub esp_device: String,
    pub deploy_device: String,
    pub deploy_fs: FsKind,
    pub extra_partitions: Vec<FormatPartitionSpec>,
    pub node_size: u32,
    pub sector_size: u32,
    pub force_wipe: bool,
}

#[derive(Debug, Clone, CTryToRust, RustToC)]
pub struct FormatPartitionSpec {
    pub device_path: String,
    pub fs_kind: FsKind,
}
