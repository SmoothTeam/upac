// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{CFree, CNew, CValidate};

use crate::CVersion;
use crate::types::{CSlice, CVec};

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CDiffFileEntry {
    pub struct_size: usize,

    pub path: CSlice,
    pub kind: u8,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CDiffPackageEntry {
    pub struct_size: usize,

    pub name: CSlice,
    pub kind: u8,
    pub version: CVersion,
    pub files: CVec<CDiffFileEntry>,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CSearchFileEntry {
    pub struct_size: usize,

    pub path: CSlice,
    pub package_name: CSlice,
    pub is_user: bool,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CDeployEntry {
    pub struct_size: usize,

    pub digest: CSlice,
    pub pinned: bool,
    pub current: bool,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CTransactionEntry {
    pub struct_size: usize,

    pub timestamp: u128,
    pub kind: u8,
    pub subject: CSlice,
    #[optional]
    pub message: CSlice,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CConfigSnapshotEntry {
    pub struct_size: usize,

    pub timestamp: u64,
    pub digest: CSlice,
    #[optional]
    pub message: CSlice,
}
