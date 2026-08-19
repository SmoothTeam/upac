// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_macro::{CFree, CNew, CValidate};

use crate::error::ErrorKind;
use crate::memory::{free_cslice, free_cvec_owning};
use crate::package::{CPackageMeta, CVersion};
use crate::types::{CSlice, CVec, check_size};
use crate::{DiffFileSource, FileDiffKind, PackageDiffKind};

#[repr(C)]
#[derive(CFree)]
pub struct CDiffPackageEntry {
    pub struct_size: usize,
    pub name: CSlice,
    pub kind: PackageDiffKind,
    pub version: CVersion,
    pub files: CVec<CDiffPrefixFileEntry>,
}

#[repr(C)]
#[derive(CFree, CValidate)]
pub struct CDiffFileEntryCommon {
    pub struct_size: usize,

    pub path: CSlice,
    pub kind: FileDiffKind,
}

#[repr(C)]
#[derive(CFree, CValidate)]
pub struct CDiffPrefixFileEntry {
    pub struct_size: usize,

    pub common: CDiffFileEntryCommon,
    pub source: DiffFileSource,
    pub package_name: CSlice,
    pub is_user: bool,
}

#[repr(C)]
#[derive(CFree, CValidate)]
pub struct CDiffConfigFileEntry {
    pub struct_size: usize,

    pub common: CDiffFileEntryCommon,
    #[optional]
    pub package_name: CSlice,
}

#[repr(C)]
#[derive(CFree, CValidate)]
pub struct CDiffUntrackedFileEntry {
    pub struct_size: usize,

    pub common: CDiffFileEntryCommon,
    pub source: DiffFileSource,
}

#[repr(C)]
#[derive(CFree, CValidate)]
pub struct CConfigCommitEntry {
    pub struct_size: usize,

    pub config_digest: CSlice,
    pub subject: CSlice,
    #[optional]
    pub message: CSlice,
}

#[repr(C)]
#[derive(CFree, CNew)]
pub struct CListConfigResponse {
    pub struct_size: usize,
    pub commits: CVec<CConfigCommitEntry>,
}

#[repr(C)]
#[derive(CFree, CNew)]
pub struct CListPackagesResponse {
    pub struct_size: usize,
    pub metas: CVec<CPackageMeta>,
}

#[repr(C)]
#[derive(CFree, CNew)]
pub struct CSearchMetaResponse {
    pub struct_size: usize,
    pub metas: CVec<CPackageMeta>,
}

#[repr(C)]
#[derive(CFree, CValidate)]
pub struct CSearchFileEntry {
    pub struct_size: usize,

    pub path: CSlice,
    pub package_name: CSlice,
    pub is_user: bool,
}

#[repr(C)]
#[derive(CFree, CNew)]
pub struct CSearchFilesResponse {
    pub struct_size: usize,
    pub files: CVec<CSearchFileEntry>,
}

#[repr(C)]
#[derive(CFree, CNew)]
pub struct CSearchInMetaResponse {
    pub struct_size: usize,
    pub metas: CVec<CPackageMeta>,
}

#[repr(C)]
#[derive(CFree, CNew)]
pub struct CSearchInPackageFilesResponse {
    pub struct_size: usize,
    pub files: CVec<CSearchFileEntry>,
}

#[repr(C)]
#[derive(CFree, CValidate)]
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
#[derive(CFree, CNew)]
pub struct CListPrefixResponse {
    pub struct_size: usize,
    pub prefixes: CVec<CPrefixEntry>,
}

#[repr(C)]
#[derive(CFree, CValidate)]
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

#[repr(C)]
#[derive(CFree, CNew)]
pub struct CListHistoryResponse {
    pub struct_size: usize,
    pub history: CVec<CHistoryEntry>,
}

#[repr(C)]
#[derive(CFree, CNew)]
pub struct CDiffPrefixResponse {
    pub struct_size: usize,
    pub files: CVec<CDiffPrefixFileEntry>,
}

#[repr(C)]
#[derive(CFree, CNew)]
pub struct CDiffConfigResponse {
    pub struct_size: usize,
    pub files: CVec<CDiffConfigFileEntry>,
}

#[repr(C)]
#[derive(CFree, CNew)]
pub struct CDiffPackagesResponse {
    pub struct_size: usize,
    pub diff_packages: CVec<CDiffPackageEntry>,
}

#[repr(C)]
#[derive(CFree, CNew)]
pub struct CDiffResponse {
    pub struct_size: usize,
    pub diff_packages: CVec<CDiffPackageEntry>,
    pub unattached_files: CVec<CDiffUntrackedFileEntry>,
}

#[repr(C)]
#[derive(CFree)]
pub struct CUnmutatedResponse {
    pub struct_size: usize,

    pub metas: CVec<CPackageMeta>,
    pub files: CVec<CDiffPrefixFileEntry>,
    pub commits: CVec<CConfigCommitEntry>,
    pub diff_packages: CVec<CDiffPackageEntry>,
}
