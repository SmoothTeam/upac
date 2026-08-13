// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::{CString, c_void};
use std::ptr::null;

use super::commit::CCommitEntry;
use super::ctypes::{CArray, CDiffKind, CSlice};
use super::file::CDiffFileEntry;
use super::packages::{CDiffPackageEntry, CPackage, CPackageInfo, CPackageMeta};
use super::{CancelToken, HookFn};

// ── CUnmutatedResponse ────────────────────────────────────────────────────────
#[repr(C)]
pub struct CUnmutatedResponse {
    struct_size: usize,
    pub metas: CArray<CPackageMeta>,
    pub files: CArray<CDiffFileEntry>,
    pub commits: CArray<CCommitEntry>,
    pub diff_packages: CArray<CDiffPackageEntry>,
}

impl CUnmutatedResponse {
    pub fn empty() -> Self {
        Self {
            struct_size: size_of::<Self>(),
            metas: CArray::empty(),
            files: CArray::empty(),
            commits: CArray::empty(),
            diff_packages: CArray::empty(),
        }
    }
}

// ── CMutatedRequest ───────────────────────────────────────────────────────────
#[repr(C)]
pub struct CMutatedRequest {
    struct_size: usize,
    repo_path: CSlice,
    root_path: CSlice,
    tmp_path: CSlice,
    branch: CSlice,
    packages: *const CPackage,
    packages_count: usize,
    uninstall_packages: *const CPackageInfo,
    uninstall_packages_len: usize,
    config_digest: CSlice,
    message: CSlice,
    files: *const CSlice,
    files_len: usize,
    file_kind: CDiffKind,
    file_package: *const CPackageInfo,
    on_hook: Option<HookFn>,
    hook_ctx: *mut c_void,
    cancel_token: *mut CancelToken,
}

impl CMutatedRequest {
    fn base(
        repo_path: &CString, root_path: &CString, branch: &CString, on_hook: Option<HookFn>, hook_ctx: *mut c_void,
        cancel_token: *mut CancelToken,
    ) -> Self {
        Self {
            struct_size: size_of::<Self>(),
            repo_path: CSlice::from_cstring(repo_path),
            root_path: CSlice::from_cstring(root_path),
            tmp_path: CSlice::empty_str(),
            branch: CSlice::from_cstring(branch),
            packages: null(),
            packages_count: 0,
            uninstall_packages: null(),
            uninstall_packages_len: 0,
            config_digest: CSlice::empty(),
            message: CSlice::empty(),
            files: null(),
            files_len: 0,
            file_kind: CDiffKind::Added,
            file_package: null(),
            on_hook,
            hook_ctx,
            cancel_token,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn for_install(
        packages: &[CPackage], repo_path: &CString, root_path: &CString, tmp_path: &CString, branch: &CString,
        on_hook: Option<HookFn>, hook_ctx: *mut c_void, cancel_token: *mut CancelToken,
    ) -> Self {
        let mut req = Self::base(repo_path, root_path, branch, on_hook, hook_ctx, cancel_token);
        req.tmp_path = CSlice::from_cstring(tmp_path);
        req.packages = if packages.is_empty() { null() } else { packages.as_ptr() };
        req.packages_count = packages.len();
        req
    }

    #[allow(clippy::too_many_arguments)]
    pub fn for_uninstall(
        package_infos: &[CPackageInfo], repo_path: &CString, root_path: &CString, branch: &CString,
        on_hook: Option<HookFn>, hook_ctx: *mut c_void, cancel_token: *mut CancelToken,
    ) -> Self {
        let mut req = Self::base(repo_path, root_path, branch, on_hook, hook_ctx, cancel_token);
        req.uninstall_packages = if package_infos.is_empty() {
            null()
        } else {
            package_infos.as_ptr()
        };
        req.uninstall_packages_len = package_infos.len();
        req
    }

    #[allow(clippy::too_many_arguments)]
    pub fn for_commit(
        message: &CString, repo_path: &CString, root_path: &CString, branch: &CString, on_hook: Option<HookFn>,
        hook_ctx: *mut c_void, cancel_token: *mut CancelToken,
    ) -> Self {
        let mut req = Self::base(repo_path, root_path, branch, on_hook, hook_ctx, cancel_token);
        req.message = CSlice::from_cstring(message);
        req
    }

    #[allow(clippy::too_many_arguments)]
    pub fn for_rollback(
        config_digest: &CString, repo_path: &CString, root_path: &CString, branch: &CString, on_hook: Option<HookFn>,
        hook_ctx: *mut c_void, cancel_token: *mut CancelToken,
    ) -> Self {
        let mut req = Self::base(repo_path, root_path, branch, on_hook, hook_ctx, cancel_token);
        req.config_digest = CSlice::from_cstring(config_digest);
        req
    }

    #[allow(clippy::too_many_arguments)]
    pub fn for_files(
        files: &[CSlice], file_kind: CDiffKind, file_package: &CPackageInfo, repo_path: &CString, root_path: &CString,
        tmp_path: &CString, branch: &CString, on_hook: Option<HookFn>, hook_ctx: *mut c_void,
        cancel_token: *mut CancelToken,
    ) -> Self {
        let mut req = Self::base(repo_path, root_path, branch, on_hook, hook_ctx, cancel_token);
        req.tmp_path = CSlice::from_cstring(tmp_path);
        req.files = if files.is_empty() { null() } else { files.as_ptr() };
        req.files_len = files.len();
        req.file_kind = file_kind;
        req.file_package = file_package as *const CPackageInfo;
        req
    }
}

// ── CUnmutatedRequest ─────────────────────────────────────────────────────────
static DUMMY_REPO_MODE: u32 = 0;

#[repr(C)]
pub struct CUnmutatedRequest {
    struct_size: usize,
    repo_path: CSlice,
    root_path: CSlice,
    tmp_path: CSlice,
    branch: CSlice,
    from_config_digest: CSlice,
    to_config_digest: CSlice,
    search: CSlice,
    symlinks: *const CSlice,
    symlinks_len: usize,
    repo_mode: *mut c_void,
    cancel_token: *mut CancelToken,
}

impl CUnmutatedRequest {
    pub fn for_list_commits(
        repo_path: &CString, root_path: &CString, branch: &CString, cancel_token: *mut CancelToken,
    ) -> Self {
        Self {
            struct_size: size_of::<Self>(),
            repo_path: CSlice::from_cstring(repo_path),
            root_path: CSlice::from_cstring(root_path),
            tmp_path: CSlice::empty(),
            branch: CSlice::from_cstring(branch),
            from_config_digest: CSlice::empty(),
            to_config_digest: CSlice::empty(),
            search: CSlice::empty(),
            symlinks: null(),
            symlinks_len: 0,
            repo_mode: &raw const DUMMY_REPO_MODE as *mut c_void,
            cancel_token,
        }
    }

    pub fn for_list_metas(root_path: &CString, cancel_token: *mut CancelToken) -> Self {
        Self {
            struct_size: size_of::<Self>(),
            repo_path: CSlice::empty(),
            root_path: CSlice::from_cstring(root_path),
            tmp_path: CSlice::empty(),
            branch: CSlice::empty(),
            from_config_digest: CSlice::empty(),
            to_config_digest: CSlice::empty(),
            search: CSlice::empty(),
            symlinks: null(),
            symlinks_len: 0,
            repo_mode: &raw const DUMMY_REPO_MODE as *mut c_void,
            cancel_token,
        }
    }

    pub fn for_diff(
        repo_path: &CString, tmp_path: &CString, from_commit: &CString, to_commit: &CString,
        cancel_token: *mut CancelToken,
    ) -> Self {
        Self {
            struct_size: size_of::<Self>(),
            repo_path: CSlice::from_cstring(repo_path),
            root_path: CSlice::empty(),
            tmp_path: CSlice::from_cstring(tmp_path),
            branch: CSlice::empty(),
            from_config_digest: CSlice::from_cstring(from_commit),
            to_config_digest: CSlice::from_cstring(to_commit),
            search: CSlice::empty(),
            symlinks: null(),
            symlinks_len: 0,
            repo_mode: &raw const DUMMY_REPO_MODE as *mut c_void,
            cancel_token,
        }
    }

    pub fn for_search(root_path: &CString, query: &CString, cancel_token: *mut CancelToken) -> Self {
        Self {
            struct_size: size_of::<Self>(),
            repo_path: CSlice::empty(),
            root_path: CSlice::from_cstring(root_path),
            tmp_path: CSlice::empty(),
            branch: CSlice::empty(),
            from_config_digest: CSlice::empty(),
            to_config_digest: CSlice::empty(),
            search: CSlice::from_cstring(query),
            symlinks: null(),
            symlinks_len: 0,
            repo_mode: &raw const DUMMY_REPO_MODE as *mut c_void,
            cancel_token,
        }
    }

    pub fn for_init(
        repo_path: &CString, root_path: &CString, branch: &CString, symlinks: &[CSlice], repo_mode_val: &u32,
        cancel_token: *mut CancelToken,
    ) -> Self {
        Self {
            struct_size: size_of::<Self>(),
            repo_path: CSlice::from_cstring(repo_path),
            root_path: CSlice::from_cstring(root_path),
            tmp_path: CSlice::empty_str(),
            branch: CSlice::from_cstring(branch),
            from_config_digest: CSlice::empty(),
            to_config_digest: CSlice::empty(),
            search: CSlice::empty(),
            symlinks: if symlinks.is_empty() { null() } else { symlinks.as_ptr() },
            symlinks_len: symlinks.len(),
            repo_mode: repo_mode_val as *const u32 as *mut c_void,
            cancel_token,
        }
    }
}

// ── CPrepareRequest ───────────────────────────────────────────────────────────
#[repr(C)]
pub struct CPrepareRequest {
    struct_size: usize,
    checksum: CSlice,
    package_path: CSlice,
    temp_dir_path: CSlice,
    on_hook: Option<HookFn>,
    hook_ctx: *mut c_void,
    cancel_token: *mut CancelToken,
}

impl CPrepareRequest {
    pub fn new(
        package_path: &CString, temp_dir_path: &CString, checksum: &CString, on_hook: Option<HookFn>,
        hook_ctx: *mut c_void, cancel_token: *mut CancelToken,
    ) -> Self {
        Self {
            struct_size: size_of::<Self>(),
            checksum: CSlice::from_cstring(checksum),
            package_path: CSlice::from_cstring(package_path),
            temp_dir_path: CSlice::from_cstring(temp_dir_path),
            on_hook,
            hook_ctx,
            cancel_token,
        }
    }
}
