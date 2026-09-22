// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{CFree, CNew, CValidate};

use super::CRequestBase;

use crate::FsKind;
use crate::error::ErrorKind;
use crate::memory::{free_cslice, free_cvec_owning};
use crate::types::{CSlice, CVec, check_size};

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CSetupPartitionRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub device_path: CSlice,
    pub esp_size_mib: u64,
    pub deploy_size_mib: u64,
    pub extra_partitions: CVec<CPartitionSpec>,
    pub force_wipe: bool,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CPartitionSpec {
    pub struct_size: usize,

    pub mount_path: CSlice,
    pub size_mib: u64,
    pub fs_kind: FsKind,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CPartitionMount {
    pub struct_size: usize,

    pub mount_path: CSlice,
    pub device_path: CSlice,
    pub fs_kind: FsKind,
}
