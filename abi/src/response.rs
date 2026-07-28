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

    pub checksum: CSlice,
    pub subject: CSlice,
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
