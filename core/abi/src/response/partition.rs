// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{CFree, CNew, CValidate};

use crate::types::CSlice;

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CSetupPartitionAddResponse {
    pub struct_size: usize,

    pub label: CSlice,
    pub device_path: CSlice,
}
