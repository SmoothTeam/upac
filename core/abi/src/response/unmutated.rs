// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{CFree, CNew, CValidate};

use super::entry::{
    CConfigCommitEntry, CDiffConfigFileEntry, CDiffPackageEntry, CDiffPrefixFileEntry, CDiffUntrackedFileEntry,
    CHistoryEntry, CPrefixEntry, CSearchFileEntry,
};

use crate::package::CPackageMeta;
use crate::types::CVec;

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CSearchMetaResponse {
    pub struct_size: usize,
    pub metas: CVec<CPackageMeta>,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CSearchFilesResponse {
    pub struct_size: usize,
    pub files: CVec<CSearchFileEntry>,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CSearchInMetaResponse {
    pub struct_size: usize,
    pub metas: CVec<CPackageMeta>,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CSearchInPackageFilesResponse {
    pub struct_size: usize,
    pub files: CVec<CSearchFileEntry>,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CListConfigResponse {
    pub struct_size: usize,
    pub commits: CVec<CConfigCommitEntry>,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CListPackagesResponse {
    pub struct_size: usize,
    pub metas: CVec<CPackageMeta>,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CListPrefixResponse {
    pub struct_size: usize,
    pub prefixes: CVec<CPrefixEntry>,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CListHistoryResponse {
    pub struct_size: usize,
    pub history: CVec<CHistoryEntry>,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CDiffPrefixResponse {
    pub struct_size: usize,
    pub files: CVec<CDiffPrefixFileEntry>,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CDiffConfigResponse {
    pub struct_size: usize,
    pub files: CVec<CDiffConfigFileEntry>,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CDiffPackagesResponse {
    pub struct_size: usize,
    pub diff_packages: CVec<CDiffPackageEntry>,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CDiffResponse {
    pub struct_size: usize,
    pub diff_packages: CVec<CDiffPackageEntry>,
    pub unattached_files: CVec<CDiffUntrackedFileEntry>,
}
