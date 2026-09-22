// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{CFree, CNew, CValidate};

use crate::types::{CSlice, CVec};

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CSetupPartitionResponse {
    pub struct_size: usize,

    pub esp_device: CSlice,
    pub deploy_device: CSlice,
    pub extra_devices: CVec<CSlice>,
}
