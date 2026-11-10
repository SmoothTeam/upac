// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{CFree, CNew, CValidate};

use super::CRequestBase;

use crate::entry::{
    CConfigSnapshotEntry, CDeployEntry, CDiffFileEntry, CDiffPackageEntry, CSearchFileEntry, CTransactionEntry,
};
use crate::types::{CSlice, CVec};
use crate::{CPackageId, CPackageMeta};

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CListPackagesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[out]
    pub metas: CVec<CPackageMeta>,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CListDeploysRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[out]
    pub deploys: CVec<CDeployEntry>,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CListHistoryRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub digest: CSlice,

    #[out]
    pub transactions: CVec<CTransactionEntry>,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CListConfigRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub digest: CSlice,

    #[out]
    pub snapshots: CVec<CConfigSnapshotEntry>,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CDiffRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub from_digest: CSlice,
    #[optional]
    pub to_digest: CSlice,

    #[out]
    pub packages: CVec<CDiffPackageEntry>,
    #[out]
    pub files: CVec<CDiffFileEntry>,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CSearchMetaRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub search: CSlice,
    pub is_regex: bool,

    #[out]
    pub metas: CVec<CPackageMeta>,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CSearchFilesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub search: CSlice,
    pub is_regex: bool,

    #[out]
    pub files: CVec<CSearchFileEntry>,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CSearchInMetaRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub package: CPackageId,
    pub search: CSlice,
    pub is_regex: bool,

    #[out]
    pub metas: CVec<CPackageMeta>,
}

#[repr(C)]
#[derive(Clone, Copy, CFree, CNew, CValidate)]
pub struct CSearchInPackageFilesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub package: CPackageId,
    pub search: CSlice,
    pub is_regex: bool,

    #[out]
    pub files: CVec<CSearchFileEntry>,
}
