// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{CFree, CNew, CValidate};

use super::CRequestBase;

use crate::FsKind;
use crate::types::{CSlice, CVec};

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CSetupFormatRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub esp_device: CSlice,
    pub deploy_device: CSlice,
    pub deploy_fs: FsKind,
    pub extra_partitions: CVec<CFormatPartitionSpec>,
    pub node_size: u32,
    pub sector_size: u32,
    pub force_wipe: bool,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CFormatPartitionSpec {
    pub struct_size: usize,

    pub device_path: CSlice,
    pub fs_kind: FsKind,
}
