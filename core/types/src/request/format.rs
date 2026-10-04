// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::request::CRequestBase;
use upac_abi::request::format::CSetupFormatRequest;
use upac_abi::types::{COwned, CSlice};

use upac_macro::{CEnum, CTryToRust, RustToC};

use super::RequestBase;
use crate::error::ErrorKind;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, CEnum)]
pub enum FsKind {
    Ext4 = 0,
    Btrfs = 1,
    Xfs = 2,
    Vfat = 3,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SetupFormatRequest<'data> {
    pub base: RequestBase<'data>,

    pub device_path: &'data str,
    pub label: Option<&'data str>,
    pub fs_kind: FsKind,
    pub require_esp: bool,
    pub force_wipe: bool,
    pub btrfs_node_size: u32,
    pub btrfs_sector_size: u32,
}
