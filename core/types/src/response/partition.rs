// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::response::partition::CSetupPartitionAddResponse;
use upac_abi::types::{COwned, CSlice};

use upac_macro::{CTryToRust, RustToC};

use crate::error::ErrorKind;

#[derive(Debug, Clone, CTryToRust, RustToC)]
pub struct SetupPartitionAddResponse {
    pub label: String,
    pub device_path: String,
}
