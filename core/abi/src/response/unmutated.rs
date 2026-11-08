// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{CFree, CNew, CValidate};

use super::entry::{
    CConfigSnapshotEntry, CDeployEntry, CDiffFileEntry, CDiffPackageEntry, CSearchFileEntry, CTransactionEntry,
};

use crate::CPackageMeta;
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
pub struct CListPackagesResponse {
    pub struct_size: usize,
    pub metas: CVec<CPackageMeta>,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CListDeploysResponse {
    pub struct_size: usize,
    pub deploys: CVec<CDeployEntry>,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CListHistoryResponse {
    pub struct_size: usize,
    pub transactions: CVec<CTransactionEntry>,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CListConfigResponse {
    pub struct_size: usize,
    pub snapshots: CVec<CConfigSnapshotEntry>,
}

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CDiffResponse {
    pub struct_size: usize,
    pub packages: CVec<CDiffPackageEntry>,
    pub files: CVec<CDiffFileEntry>,
}
