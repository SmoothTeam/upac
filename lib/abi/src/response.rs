// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use upac_macro::{CFree, CValidate};

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
pub struct CListConfigResponse {
    pub struct_size: usize,
    pub commits: CVec<CConfigCommitEntry>,
}

impl CListConfigResponse {
    /// # Safety
    /// Must be called at most once. Assumes every buffer reachable from `self` was allocated by
    /// this library (via `CVec::from_owned`/`CSlice::from_owned`), not hand-constructed by the caller.
    pub unsafe fn free(&self) {
        unsafe { free_cvec_owning(&self.commits, |entry| entry.free()) };
    }
}

#[repr(C)]
pub struct CListPackagesResponse {
    pub struct_size: usize,
    pub metas: CVec<CPackageMeta>,
}

impl CListPackagesResponse {
    /// # Safety
    /// Must be called at most once. Assumes every buffer reachable from `self` was allocated by
    /// this library (via `CVec::from_owned`/`CSlice::from_owned`), not hand-constructed by the caller.
    pub unsafe fn free(&self) {
        unsafe { free_cvec_owning(&self.metas, |meta| meta.free()) };
    }
}

#[repr(C)]
pub struct CSearchMetaResponse {
    pub struct_size: usize,
    pub metas: CVec<CPackageMeta>,
}

impl CSearchMetaResponse {
    /// # Safety
    /// Must be called at most once. Assumes every buffer reachable from `self` was allocated by
    /// this library (via `CVec::from_owned`/`CSlice::from_owned`), not hand-constructed by the caller.
    pub unsafe fn free(&self) {
        unsafe { free_cvec_owning(&self.metas, |meta| meta.free()) };
    }
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
pub struct CSearchFilesResponse {
    pub struct_size: usize,
    pub files: CVec<CSearchFileEntry>,
}

impl CSearchFilesResponse {
    /// # Safety
    /// Must be called at most once. Assumes every buffer reachable from `self` was allocated by
    /// this library (via `CVec::from_owned`/`CSlice::from_owned`), not hand-constructed by the caller.
    pub unsafe fn free(&self) {
        unsafe { free_cvec_owning(&self.files, |entry| entry.free()) };
    }
}

#[repr(C)]
pub struct CSearchInMetaResponse {
    pub struct_size: usize,
    pub metas: CVec<CPackageMeta>,
}

impl CSearchInMetaResponse {
    /// # Safety
    /// Must be called at most once. Assumes every buffer reachable from `self` was allocated by
    /// this library (via `CVec::from_owned`/`CSlice::from_owned`), not hand-constructed by the caller.
    pub unsafe fn free(&self) {
        unsafe { free_cvec_owning(&self.metas, |meta| meta.free()) };
    }
}

#[repr(C)]
pub struct CSearchInPackageFilesResponse {
    pub struct_size: usize,
    pub files: CVec<CSearchFileEntry>,
}

impl CSearchInPackageFilesResponse {
    /// # Safety
    /// Must be called at most once. Assumes every buffer reachable from `self` was allocated by
    /// this library (via `CVec::from_owned`/`CSlice::from_owned`), not hand-constructed by the caller.
    pub unsafe fn free(&self) {
        unsafe { free_cvec_owning(&self.files, |entry| entry.free()) };
    }
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
pub struct CListPrefixResponse {
    pub struct_size: usize,
    pub prefixes: CVec<CPrefixEntry>,
}

impl CListPrefixResponse {
    /// # Safety
    /// Must be called at most once. Assumes every buffer reachable from `self` was allocated by
    /// this library (via `CVec::from_owned`/`CSlice::from_owned`), not hand-constructed by the caller.
    pub unsafe fn free(&self) {
        unsafe { free_cvec_owning(&self.prefixes, |entry| entry.free()) };
    }
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
pub struct CListHistoryResponse {
    pub struct_size: usize,
    pub history: CVec<CHistoryEntry>,
}

impl CListHistoryResponse {
    /// # Safety
    /// Must be called at most once. Assumes every buffer reachable from `self` was allocated by
    /// this library (via `CVec::from_owned`/`CSlice::from_owned`), not hand-constructed by the caller.
    pub unsafe fn free(&self) {
        unsafe { free_cvec_owning(&self.history, |entry| entry.free()) };
    }
}

#[repr(C)]
pub struct CDiffPrefixResponse {
    pub struct_size: usize,
    pub files: CVec<CDiffPrefixFileEntry>,
}

impl CDiffPrefixResponse {
    /// # Safety
    /// Must be called at most once. Assumes every buffer reachable from `self` was allocated by
    /// this library (via `CVec::from_owned`/`CSlice::from_owned`), not hand-constructed by the caller.
    pub unsafe fn free(&self) {
        unsafe { free_cvec_owning(&self.files, |entry| entry.free()) };
    }
}

#[repr(C)]
pub struct CDiffConfigResponse {
    pub struct_size: usize,
    pub files: CVec<CDiffConfigFileEntry>,
}

impl CDiffConfigResponse {
    /// # Safety
    /// Must be called at most once. Assumes every buffer reachable from `self` was allocated by
    /// this library (via `CVec::from_owned`/`CSlice::from_owned`), not hand-constructed by the caller.
    pub unsafe fn free(&self) {
        unsafe { free_cvec_owning(&self.files, |entry| entry.free()) };
    }
}

#[repr(C)]
pub struct CDiffPackagesResponse {
    pub struct_size: usize,
    pub diff_packages: CVec<CDiffPackageEntry>,
}

impl CDiffPackagesResponse {
    /// # Safety
    /// Must be called at most once. Assumes every buffer reachable from `self` was allocated by
    /// this library (via `CVec::from_owned`/`CSlice::from_owned`), not hand-constructed by the caller.
    pub unsafe fn free(&self) {
        unsafe { free_cvec_owning(&self.diff_packages, |entry| entry.free()) };
    }
}

#[repr(C)]
pub struct CDiffResponse {
    pub struct_size: usize,
    pub diff_packages: CVec<CDiffPackageEntry>,
    pub unattached_files: CVec<CDiffUntrackedFileEntry>,
}

impl CDiffResponse {
    /// # Safety
    /// Must be called at most once. Assumes every buffer reachable from `self` was allocated by
    /// this library (via `CVec::from_owned`/`CSlice::from_owned`), not hand-constructed by the caller.
    pub unsafe fn free(&self) {
        unsafe {
            free_cvec_owning(&self.unattached_files, |entry| entry.free());
            free_cvec_owning(&self.diff_packages, |entry| entry.free());
        }
    }
}

#[repr(C)]
pub struct CUnmutatedResponse {
    pub struct_size: usize,

    pub metas: CVec<CPackageMeta>,
    pub files: CVec<CDiffPrefixFileEntry>,
    pub commits: CVec<CConfigCommitEntry>,
    pub diff_packages: CVec<CDiffPackageEntry>,
}

impl CUnmutatedResponse {
    /// # Safety
    /// Must be called at most once. Assumes every buffer reachable from `self` was allocated by
    /// this library (via `CVec::from_owned`/`CSlice::from_owned`), not hand-constructed by the caller.
    pub unsafe fn free(&self) {
        unsafe {
            free_cvec_owning(&self.metas, |package_meta_c| package_meta_c.free());
            free_cvec_owning(&self.files, |diff_file_entry_c| diff_file_entry_c.free());
            free_cvec_owning(&self.commits, |commit_entry_c| commit_entry_c.free());
            free_cvec_owning(&self.diff_packages, |diff_package_entry_c| diff_package_entry_c.free());
        }
    }
}
