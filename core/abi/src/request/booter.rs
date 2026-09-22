// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{CFree, CNew, CValidate};

use crate::error::ErrorKind;
use crate::memory::free_cslice;
use crate::types::{CSlice, check_size};

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CBootPluginSetOneShotRequest {
    pub struct_size: usize,

    pub entry_name: CSlice,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CBootPluginConfirmSuccsesBootRequest {
    pub struct_size: usize,

    pub entry_name: CSlice,
    pub esp_mount_point: CSlice,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CBootPluginInstallRequest {
    pub struct_size: usize,

    pub esp_mount_point: CSlice,
    pub esp_partition_number: u32,
    pub esp_starting_lba: u64,
    pub esp_ending_lba: u64,
    pub esp_unique_partition_guid: [u8; 16],
    pub to_slot: CSlice,
    pub from_slot: CSlice,
}
