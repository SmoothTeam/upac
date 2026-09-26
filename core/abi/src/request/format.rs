// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{CEnum, CFree, CNew, CValidate};

use super::CRequestBase;

use crate::types::CSlice;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, CEnum)]
pub enum FsKind {
    Ext4 = 0,
    Btrfs = 1,
    Xfs = 2,
    Vfat = 3,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CSetupFormatRequest {
    pub struct_size: usize,

    pub base: CRequestBase,

    pub device_path: CSlice,
    #[optional]
    pub label: CSlice,
    pub fs_kind: u8,
    pub require_esp: bool,
    pub force_wipe: bool,
    pub btrfs_node_size: u32,
    pub btrfs_sector_size: u32,
}
