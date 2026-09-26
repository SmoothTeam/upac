// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::ErrorKind;
use upac_abi::request::CRequestBase;
use upac_abi::request::format::CSetupFormatRequest;
use upac_abi::request::format::FsKind;
use upac_abi::types::{COwned, CSlice};

use upac_macro::{CTryToRust, RustToC};

use super::RequestBase;

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SetupFormatRequest<'data> {
    pub base: RequestBase,

    pub device_path: &'data str,
    pub label: Option<&'data str>,
    pub fs_kind: FsKind,
    pub require_esp: bool,
    pub force_wipe: bool,
    pub btrfs_node_size: u32,
    pub btrfs_sector_size: u32,
}
