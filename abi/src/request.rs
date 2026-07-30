use std::os::raw::c_void;

use upac_macro::CValidate;

use crate::DiffKind;
use crate::error::ErrorKind;
use crate::hook::{HookCancelToken, HookMessageFn};
use crate::package::{CPackageInfo, CUnpackedPackage};
use crate::types::{CBorrowed, CSlice, CVec, check_size};

#[repr(C)]
#[derive(CValidate)]
pub struct CRequestBase {
    pub struct_size: usize,

    pub branch: CSlice,

    pub on_hook: Option<HookMessageFn>,
    pub hook_ctx: *mut c_void,

    pub hook_cancel_token: *mut HookCancelToken,
}

#[repr(C)]
pub struct CInstallRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub tmp_path: CSlice,

    pub subject: CSlice,
    pub message: CSlice,

    pub packages: CVec<CUnpackedPackage>,
}

impl CInstallRequest {
    pub unsafe fn validate(&self) -> Result<(), ErrorKind> {
        check_size::<CInstallRequest>(self.struct_size)?;
        unsafe { self.base.validate()? };
        unsafe { self.tmp_path.validate()? };
        unsafe { self.subject.validate()? };
        if !self.message.ptr.is_null() {
            unsafe { self.message.validate()? };
        }
        unsafe { self.packages.validate()? };

        for package in unsafe { self.packages.as_slice() } {
            unsafe { package.validate()? };
        }

        Ok(())
    }
}

#[repr(C)]
pub struct CUpdateRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub tmp_path: CSlice,

    pub subject: CSlice,
    pub message: CSlice,

    pub packages: CVec<CUnpackedPackage>,
}

impl CUpdateRequest {
    pub unsafe fn validate(&self) -> Result<(), ErrorKind> {
        check_size::<CUpdateRequest>(self.struct_size)?;
        unsafe { self.base.validate()? };
        unsafe { self.tmp_path.validate()? };
        unsafe { self.subject.validate()? };
        if !self.message.ptr.is_null() {
            unsafe { self.message.validate()? };
        }
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

    pub tmp_path: CSlice,
    pub subject: CSlice,
    pub message: CSlice,
    pub packages: CVec<CPackageInfo>,
}

impl CUninstallRequest {
    pub unsafe fn validate(&self) -> Result<(), ErrorKind> {
        check_size::<CUninstallRequest>(self.struct_size)?;
        unsafe { self.base.validate()? };
        unsafe { self.tmp_path.validate()? };
        unsafe { self.subject.validate()? };
        if !self.message.ptr.is_null() {
            unsafe { self.message.validate()? };
        }
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

    pub tmp_path: CSlice,
    pub commit_hash: CSlice,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CCommitRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub tmp_path: CSlice,
    pub subject: CSlice,
    #[optional]
    pub message: CSlice,
}

#[repr(C)]
pub struct CFilesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub tmp_path: CSlice,
    pub subject: CSlice,
    pub message: CSlice,
    pub files: CVec<CSlice>,
    pub file_kind: DiffKind,
    pub file_package: *const CPackageInfo,
}

impl CFilesRequest {
    pub unsafe fn validate(&self) -> Result<(), ErrorKind> {
        check_size::<CFilesRequest>(self.struct_size)?;
        unsafe { self.base.validate()? };
        unsafe { self.tmp_path.validate()? };
        unsafe { self.subject.validate()? };
        if !self.message.ptr.is_null() {
            unsafe { self.message.validate()? };
        }

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
#[derive(CValidate)]
pub struct CListPackagesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CListCommitRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub prefix_digest: CSlice,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CListPrefixRequest {
    pub struct_size: usize,
    pub base: CRequestBase,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CListHistoryRequest {
    pub struct_size: usize,
    pub base: CRequestBase,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CDiffFilesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub from_commit_hash: CSlice,
    #[optional]
    pub to_commit_hash: CSlice,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CDiffPackagesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub from_commit_hash: CSlice,
    #[optional]
    pub to_commit_hash: CSlice,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CDiffRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    #[optional]
    pub from_commit_hash: CSlice,
    #[optional]
    pub to_commit_hash: CSlice,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CSearchMetaRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub search: CSlice,
}

#[repr(C)]
#[derive(CValidate)]
pub struct CSearchFilesRequest {
    pub struct_size: usize,
    pub base: CRequestBase,

    pub search: CSlice,
}
