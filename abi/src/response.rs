use upac_macro::{CFree, CValidate};

use crate::DiffKind;
use crate::error::ErrorKind;
use crate::memory::{free_cslice, free_cvec_owning};
use crate::package::{CPackageMeta, CVersion};
use crate::types::{CSlice, CVec, check_size};

#[repr(C)]
#[derive(CFree)]
pub struct CDiffPackageEntry {
    pub struct_size: usize,
    pub name: CSlice,
    pub kind: DiffKind,
    pub version: CVersion,
}

#[repr(C)]
#[derive(CFree, CValidate)]
pub struct CDiffFileEntry {
    pub struct_size: usize,

    pub path: CSlice,
    pub kind: DiffKind,
    pub package_name: CSlice,
    pub is_user: bool,
}

#[repr(C)]
#[derive(CFree, CValidate)]
pub struct CCommitEntry {
    pub struct_size: usize,

    pub config_digest: CSlice,
    pub subject: CSlice,
    #[optional]
    pub message: CSlice,
}

#[repr(C)]
pub struct CListCommitResponse {
    pub struct_size: usize,
    pub commits: CVec<CCommitEntry>,
}

impl CListCommitResponse {
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
    pub config_history: CVec<CCommitEntry>,
}

#[repr(C)]
pub struct CListHistoryResponse {
    pub struct_size: usize,
    pub history: CVec<CHistoryEntry>,
}

impl CListHistoryResponse {
    pub unsafe fn free(&self) {
        unsafe { free_cvec_owning(&self.history, |entry| entry.free()) };
    }
}

#[repr(C)]
pub struct CDiffFilesResponse {
    pub struct_size: usize,
    pub files: CVec<CDiffFileEntry>,
}

impl CDiffFilesResponse {
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
    pub unsafe fn free(&self) {
        unsafe { free_cvec_owning(&self.diff_packages, |entry| entry.free()) };
    }
}

#[repr(C)]
pub struct CDiffResponse {
    pub struct_size: usize,
    pub files: CVec<CDiffFileEntry>,
    pub diff_packages: CVec<CDiffPackageEntry>,
}

impl CDiffResponse {
    pub unsafe fn free(&self) {
        unsafe {
            free_cvec_owning(&self.files, |entry| entry.free());
            free_cvec_owning(&self.diff_packages, |entry| entry.free());
        }
    }
}

#[repr(C)]
pub struct CUnmutatedResponse {
    pub struct_size: usize,

    pub metas: CVec<CPackageMeta>,
    pub files: CVec<CDiffFileEntry>,
    pub commits: CVec<CCommitEntry>,
    pub diff_packages: CVec<CDiffPackageEntry>,
}

impl CUnmutatedResponse {
    pub unsafe fn free(&self) {
        unsafe {
            free_cvec_owning(&self.metas, |package_meta_c| package_meta_c.free());
            free_cvec_owning(&self.files, |diff_file_entry_c| diff_file_entry_c.free());
            free_cvec_owning(&self.commits, |commit_entry_c| commit_entry_c.free());
            free_cvec_owning(&self.diff_packages, |diff_package_entry_c| diff_package_entry_c.free());
        }
    }
}
