// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::ErrorKind;
use upac_abi::request::booter::{
    CBootPluginConfirmSuccessBootRequest, CBootPluginInstallRequest, CBootPluginSetOneShotRequest,
};
use upac_abi::types::{COwned, CSlice};

use upac_macro::{CTryToRust, RustToC};

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct BootPluginSetOneShotRequest {
    pub entry_name: String,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct BootPluginConfirmSuccessBootRequest {
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
