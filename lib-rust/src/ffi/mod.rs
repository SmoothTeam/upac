use std::cell::UnsafeCell;
use std::hint::spin_loop;
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicU8, Ordering};

use upac_derive::CFree;

use self::primitives::{free_carray, free_carray_owning, free_cslice};

pub use self::primitives::{AbiError, CArray, CSlice, check_size};

mod primitives;

pub const ABI_VERSION: u32 = 2;

// ── HookCancelToken ─────────────────────────────────────────────────────────
#[repr(C)]
pub struct HookCancelToken {
    cancelled: AtomicU8,
    binding_lock: AtomicU8,

    hook_cancel: UnsafeCell<Option<unsafe extern "C" fn(ctx: *mut c_void)>>,
    hook_cancel_context: UnsafeCell<*mut c_void>,
}

unsafe impl Sync for HookCancelToken {}

impl HookCancelToken {
    pub fn bind(&self, hook_cancel: unsafe extern "C" fn(ctx: *mut c_void), hook_cancel_context: *mut c_void) {
        self.lock_binding();

        unsafe {
            *self.hook_cancel.get() = Some(hook_cancel);
            *self.hook_cancel_context.get() = hook_cancel_context;
        }

        self.unlock_binding();
    }

    pub fn cancel(&self) {
        self.cancelled.store(1, Ordering::Release);

        self.lock_binding();

        let hook_cancel = unsafe { *self.hook_cancel.get() };
        let hook_cancel_context = unsafe { *self.hook_cancel_context.get() };

        self.unlock_binding();

        if let Some(hook_cancel) = hook_cancel {
            unsafe { hook_cancel(hook_cancel_context) };
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire) != 0
    }

    pub fn reset(&self) {
        self.lock_binding();

        unsafe {
            *self.hook_cancel.get() = None;
            *self.hook_cancel_context.get() = null_mut();
        }

        self.unlock_binding();

        self.cancelled.store(0, Ordering::Release);
    }

    fn lock_binding(&self) {
        while self
            .binding_lock
            .compare_exchange_weak(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            spin_loop();
        }
    }

    fn unlock_binding(&self) {
        self.binding_lock.store(0, Ordering::Release);
    }
}

// ── CVersion ────────────────────────────────────────────────────────────────
#[derive(CFree)]
#[repr(C)]
pub struct CVersion {
    pub struct_size: usize,

    pub epoch: u32,
    pub release: u32,
    pub parts: CArray<u32>,
    pub pre: CSlice,
}

impl CVersion {
    pub unsafe fn validate(&self) -> Result<(), AbiError> {
        check_size::<CVersion>(self.struct_size)?;

        unsafe {
            if self.parts.len == 0 {
                return Err(AbiError::InvalidEntry);
            }

            if !self.pre.ptr.is_null() {
                self.pre.validate()?;
            }
        }

        Ok(())
    }
}

// ── CPackageMeta ────────────────────────────────────────────────────────────
#[derive(CFree)]
#[repr(C)]
pub struct CPackageMeta {
    pub struct_size: usize,
    pub name: CSlice,
    pub version: CVersion,
    pub arch: CSlice,
    pub arch_sub: CSlice,
    pub maintainer: CSlice,
    pub description: CSlice,
    pub license: CSlice,
    pub url: CSlice,
    pub sha256: [u8; 32],
    pub installed_size: u64,
}

impl CPackageMeta {
    pub unsafe fn validate(&self) -> Result<(), AbiError> {
        check_size::<CPackageMeta>(self.struct_size)?;

        unsafe {
            self.name.validate()?;
            self.arch.validate()?;
            self.maintainer.validate()?;
            self.description.validate()?;
            self.license.validate()?;
            self.url.validate()?;
            self.version.validate()?;
        }

        if !self.arch_sub.ptr.is_null() {
            unsafe { self.arch_sub.validate()? };
        }
        Ok(())
    }
}

// ── CPackage ────────────────────────────────────────────────────────────────
#[repr(C)]
pub struct CPackage {
    pub struct_size: usize,
    pub meta: *mut CPackageMeta,
    pub temp_path: CSlice,
}

impl CPackage {
    pub unsafe fn validate(&self) -> Result<(), AbiError> {
        check_size::<CPackage>(self.struct_size)?;

        unsafe {
            (*self.meta).validate()?;
            self.temp_path.validate()?;
        };
        Ok(())
    }
}

// ── CPackageInfo ────────────────────────────────────────────────────────────
#[repr(C)]
pub struct CPackageInfo {
    pub struct_size: usize,
    pub name: CSlice,
    pub arch: CSlice,
    pub arch_sub: CSlice,
}

impl CPackageInfo {
    pub unsafe fn validate(&self) -> Result<(), AbiError> {
        check_size::<CPackageInfo>(self.struct_size)?;

        unsafe {
            self.name.validate()?;
            self.arch.validate()?;
        };
        Ok(())
    }
}

// ── HookFn / HookResponse ───────────────────────────────────────────────────
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookResponse {
    Proceed = 0,
    Cancel = 1,
}

pub type HookFn = unsafe extern "C" fn(event: u32, data: *const c_void, ctx: *mut c_void) -> HookResponse;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffKind {
    Added = 0,
    Removed = 1,
    Modified = 2,
}

impl DiffKind {
    pub fn from_u8(version: u8) -> Result<DiffKind, AbiError> {
        match version {
            0 => Ok(DiffKind::Added),
            1 => Ok(DiffKind::Removed),
            2 => Ok(DiffKind::Modified),
            _ => Err(AbiError::InvalidEntry),
        }
    }
}

// ── CMutatedRequest ─────────────────────────────────────────────────────────
#[repr(C)]
pub struct CMutatedRequest {
    pub struct_size: usize,

    pub repo_path: CSlice,
    pub root_path: CSlice,
    pub tmp_path: CSlice,
    pub branch: CSlice,

    // Install
    pub packages: *const CPackage,
    pub packages_count: usize,

    // Uninstall
    pub uninstall_packages: *const CPackageInfo,
    pub uninstall_packages_len: usize,

    // Rollback
    pub commit_hash: CSlice,

    // Commit
    pub message: CSlice,

    // Files
    pub files: *const CSlice,
    pub files_len: usize,
    pub file_kind: DiffKind,
    pub file_package: *const CPackageInfo,

    pub on_hook: Option<HookFn>,
    pub hook_ctx: *mut c_void,

    pub hook_cancel_token: *mut HookCancelToken,
}

impl CMutatedRequest {
    pub unsafe fn validate(&self) -> Result<(), AbiError> {
        check_size::<CMutatedRequest>(self.struct_size)?;

        unsafe {
            self.repo_path.validate()?;
            self.root_path.validate()?;
            self.tmp_path.validate()?;
            self.branch.validate()?;
        }
        Ok(())
    }
}

// ── CUnmutatedRequest ───────────────────────────────────────────────────────
#[repr(C)]
pub struct CUnmutatedRequest {
    pub struct_size: usize,

    pub repo_path: CSlice,
    pub root_path: CSlice,
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

impl CUnmutatedRequest {
    pub unsafe fn validate(&self) -> Result<(), AbiError> {
        check_size::<CUnmutatedRequest>(self.struct_size)?;

        unsafe {
            self.repo_path.validate()?;
            self.root_path.validate()?;
            self.tmp_path.validate()?;
            self.branch.validate()?;
        };
        Ok(())
    }
}

// ── CHookPreInstall ─────────────────────────────────────────────────────────
#[repr(C)]
pub struct CHookPreInstall {
    pub packages_count: u32,
    pub required_space: u64,
    pub free_space: u64,
}

// ── CDiffPackageEntry ───────────────────────────────────────────────────────
#[derive(CFree)]
#[repr(C)]
pub struct CDiffPackageEntry {
    pub struct_size: usize,
    pub name: CSlice,
    pub kind: DiffKind,
    pub version: CVersion,
}

// ── CDiffFileEntry ──────────────────────────────────────────────────────────
#[derive(CFree)]
#[repr(C)]
pub struct CDiffFileEntry {
    pub struct_size: usize,

    pub path: CSlice,
    pub kind: DiffKind,
    pub package_name: CSlice,
    pub is_user: bool,
}

impl CDiffFileEntry {
    pub unsafe fn validate(&self) -> Result<(), AbiError> {
        check_size::<CDiffFileEntry>(self.struct_size)?;

        Ok(())
    }
}

// ── CCommitEntry ────────────────────────────────────────────────────────────
#[derive(CFree)]
#[repr(C)]
pub struct CCommitEntry {
    pub struct_size: usize,

    pub checksum: CSlice,
    pub subject: CSlice,
}

impl CCommitEntry {
    pub unsafe fn validate(&self) -> Result<(), AbiError> {
        check_size::<CCommitEntry>(self.struct_size)?;

        Ok(())
    }
}

// ── CUnmutatedResponse ──────────────────────────────────────────────────────
#[repr(C)]
pub struct CUnmutatedResponse {
    pub struct_size: usize,

    pub metas: CArray<CPackageMeta>,
    pub files: CArray<CDiffFileEntry>,
    pub commits: CArray<CCommitEntry>,
    pub diff_packages: CArray<CDiffPackageEntry>,
}

impl CUnmutatedResponse {
    pub unsafe fn free(&self) {
        unsafe {
            free_carray_owning(&self.metas, |package_meta_c| package_meta_c.free());
            free_carray_owning(&self.files, |diff_file_entry_c| diff_file_entry_c.free());
            free_carray_owning(&self.commits, |commit_entry_c| commit_entry_c.free());
            free_carray_owning(&self.diff_packages, |diff_package_entry_c| diff_package_entry_c.free());
        }
    }
}

// ── CRepoMode ───────────────────────────────────────────────────────────────
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CRepoMode {
    Archive = 0,
    Bare = 1,
    BareUser = 2,
}
