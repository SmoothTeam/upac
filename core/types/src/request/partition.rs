// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::ErrorKind;
use upac_abi::request::CRequestBase;
use upac_abi::request::partition::PartitionKind;
use upac_abi::request::partition::{CSetupPartitionAddRequest, CSetupPartitionTableRequest};
use upac_abi::types::{COwned, CSlice};

use upac_macro::{CTryToRust, RustToC};

use super::RequestBase;

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SetupPartitionTableRequest<'data> {
    pub base: RequestBase,

    pub device_path: &'data str,
    pub force_wipe: bool,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SetupPartitionAddRequest<'data> {
    pub base: RequestBase,

    pub device_path: &'data str,
    pub label: &'data str,
    pub size_mib: u64,
    pub kind: PartitionKind,
}
