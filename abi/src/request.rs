use std::os::raw::c_void;

use upac_macro::{CFree, CValidate};

use crate::DiffKind;
use crate::error::ErrorKind;
use crate::hook::{HookCancelToken, HookMessageFn};
use crate::memory::{free_cslice, free_cvec_owning};
use crate::package::{CPackageInfo, CPackageMeta, CUnpackedPackage, CVersion};
use crate::types::{CBorrowed, CSlice, CVec, check_size};

#[repr(C)]
#[derive(CValidate)]
pub struct CRequestBase {
    pub struct_size: usize,

    pub tmp_path: CSlice,
    pub branch: CSlice,

    pub on_hook: Option<HookMessageFn>,
    pub hook_ctx: *mut c_void,

    pub hook_cancel_token: *mut HookCancelToken,
}

#[repr(C)]
pub struct CInstallRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub packages: CVec<CUnpackedPackage>,
}

impl CInstallRequest {
    pub unsafe fn validate(&self) -> Result<(), ErrorKind> {
        check_size::<CInstallRequest>(self.struct_size)?;
        unsafe { self.base.validate()? };
        unsafe { self.packages.validate()? };

        for package in unsafe { self.packages.as_slice() } {
            unsafe { package.validate()? };
        }

        Ok(())
    }
}

#[repr(C)]
pub struct CUninstallRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub packages: CVec<CPackageInfo>,
}

impl CUninstallRequest {
    pub unsafe fn validate(&self) -> Result<(), ErrorKind> {
        check_size::<CUninstallRequest>(self.struct_size)?;
        unsafe { self.base.validate()? };
        unsafe { self.packages.validate()? };

        for package in unsafe { self.packages.as_slice() } {
            unsafe { package.validate()? };
        }

        Ok(())
    }
}

#[repr(C)]
#[derive(CValidate)]
pub struct CRollbackRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub commit_hash: CSlice,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CCommitRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub message: CSlice,
}

#[repr(C)]
pub struct CFilesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub files: CVec<CSlice>,
    pub file_kind: DiffKind,
    pub file_package: *const CPackageInfo,
}

impl CFilesRequest {
    pub unsafe fn validate(&self) -> Result<(), ErrorKind> {
        check_size::<CFilesRequest>(self.struct_size)?;
        unsafe { self.base.validate()? };

        for file in unsafe { self.files.as_borrowed() } {
            unsafe { file.validate()? };
        }

        if self.file_package.is_null() {
            return Err(ErrorKind::InvalidEntry);
        }
        unsafe { (*self.file_package).validate()? };

        Ok(())
    }
}

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

#[repr(C)]
#[deprecated]
pub struct CUnmutatedRequest {
    pub struct_size: usize,

    pub tmp_path: CSlice,
    pub branch: CSlice,

    pub from_commit_hash: CSlice,
    pub to_commit_hash: CSlice,

    pub search: CSlice,

    pub symlinks: *const CSlice,
    pub symlinks_len: usize,

    pub repo_mode: *mut c_void,

    pub hook_cancel_token: *mut HookCancelToken,
}

#[allow(deprecated)]
impl CUnmutatedRequest {
    pub unsafe fn validate(&self) -> Result<(), ErrorKind> {
        check_size::<CUnmutatedRequest>(self.struct_size)?;

        unsafe {
            self.tmp_path.validate()?;
            self.branch.validate()?;
        };
        Ok(())
    }
}
