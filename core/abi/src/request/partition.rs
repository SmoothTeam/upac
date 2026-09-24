// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{CFree, CNew, CValidate};

use super::CRequestBase;

use crate::PartitionKind;
use crate::types::CSlice;

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
    pub kind: PartitionKind,
}
