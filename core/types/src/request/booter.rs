// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::request::booter::{
    CBootPluginConfirmSuccessBootRequest, CBootPluginInstallRequest, CBootPluginSetOneShotRequest,
};
use upac_abi::types::{COwned, CSlice};

use upac_macro::{CTryToRust, RustToC};

use crate::error::ErrorKind;

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

    pub to_slot: String,
    pub from_slot: String,
}
