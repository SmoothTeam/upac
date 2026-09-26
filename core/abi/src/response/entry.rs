// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{CEnum, CFree, CNew, CValidate};

use crate::package::CVersion;
use crate::types::{CSlice, CVec};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, CEnum)]
pub enum FileDiffKind {
    Added = 0,
    Removed = 1,
    Modified = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, CEnum)]
pub enum PackageDiffKind {
    Added = 0,
    Removed = 1,
    Modified = 2,
    FilesChanged = 3,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, CEnum)]
pub enum DiffFileSource {
    Prefix = 0,
    Config = 1,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CDiffPackageEntry {
    pub struct_size: usize,

    pub name: CSlice,
    pub kind: u8,
    pub version: CVersion,
    pub files: CVec<CDiffPrefixFileEntry>,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CDiffFileCommonEntry {
    pub struct_size: usize,

    pub path: CSlice,
    pub kind: u8,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CDiffPrefixFileEntry {
    pub struct_size: usize,

    pub common: CDiffFileCommonEntry,
    pub source: u8,
    pub package_name: CSlice,
    pub is_user: bool,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CDiffConfigFileEntry {
    pub struct_size: usize,

    pub common: CDiffFileCommonEntry,
    #[optional]
    pub package_name: CSlice,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CDiffUntrackedFileEntry {
    pub struct_size: usize,

    pub common: CDiffFileCommonEntry,
    pub source: u8,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CConfigCommitEntry {
    pub struct_size: usize,

    pub config_digest: CSlice,
    pub subject: CSlice,
    #[optional]
    pub message: CSlice,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CSearchFileEntry {
    pub struct_size: usize,

    pub path: CSlice,
    pub package_name: CSlice,
    pub is_user: bool,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CPrefixEntry {
    pub struct_size: usize,

    pub prefix_digest: CSlice,
    pub subject: CSlice,
    #[optional]
    pub message: CSlice,
    pub timestamp: u64,
    #[optional]
    pub working_config: CSlice,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CHistoryEntry {
    pub struct_size: usize,

    pub prefix_digest: CSlice,
    pub subject: CSlice,
    #[optional]
    pub message: CSlice,
    pub timestamp: u64,
    #[optional]
    pub working_config: CSlice,
    pub config_history: CVec<CConfigCommitEntry>,
}
