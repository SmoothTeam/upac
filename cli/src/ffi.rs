// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;

use libloading::Library;

use std::ffi::{c_void, CStr, CString};
use std::marker::PhantomData;
use std::ptr::{null, null_mut};
use std::slice;
use std::str;
use std::sync::atomic::{AtomicU8, Ordering};

pub trait Validate {
    fn validate(&self) -> Result<()>;
}

// ── CSlice ────────────────────────────────────────────────────────────────────
// (ptr, len) pair mirroring Zig's CSlice. ptr[len] MUST be 0.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CSlice {
    pub ptr: *const u8,
    pub len: usize,
}

impl CSlice {
    pub const fn empty() -> Self {
        Self {
            ptr: null(),
            len: 0,
        }
    }

    pub fn from_cstring(source: &CString) -> Self {
        let bytes = source.as_bytes();
        Self {
            ptr: bytes.as_ptr(),
            len: bytes.len(),
        }
    }

    pub fn from_cstr(source: &CStr) -> Self {
        let bytes = source.to_bytes();
        Self {
            ptr: bytes.as_ptr(),
            len: bytes.len(),
        }
    }

    /// Converts the CSlice to a Rust string slice.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// 1. The `ptr` is valid for reads for `len` bytes and is non-null.
    /// 2. The memory pointed to by `ptr` contains valid UTF-8 data.
    /// 3. The memory must not be mutated for the duration of the returned lifetime.
    pub unsafe fn as_str(&self) -> &str {
        let bytes = slice::from_raw_parts(self.ptr, self.len);
        str::from_utf8_unchecked(bytes)
    }
}

impl Validate for CSlice {
    /// Checks if the slice is valid and null-terminated.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the `ptr` is valid for reads at least up to `len + 1`
    /// bytes to safely check for the null terminator.
    fn validate(&self) -> Result<()> {
        if self.ptr.is_null() || self.len == 0 {
            return Err(anyhow::anyhow!("empty slice"));
        }
        // Safety: offset check requires ptr to be valid for len + 1
        if unsafe { *self.ptr.add(self.len) } != 0 {
            return Err(anyhow::anyhow!("not null-terminated"));
        }
        Ok(())
    }
}

// ── CArray<T> ─────────────────────────────────────────────────────────────────
#[repr(C)]
pub struct CArray<T> {
    pub ptr: *mut T,
    pub len: usize,
    _marker: PhantomData<T>,
}

impl<T> CArray<T> {
    pub const fn empty() -> Self {
        Self {
            ptr: null_mut(),
            len: 0,
            _marker: PhantomData,
        }
    }
    /// Returns a Rust slice pointing to the array's memory.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// 1. The `ptr` is valid for reads for `len * size_of::<T>()` bytes.
    /// 2. The memory is properly aligned for type `T`.
    /// 3. The data must not be modified while the returned slice is in use.
    pub unsafe fn as_slice(&self) -> &[T] {
        if self.ptr.is_null() || self.len == 0 {
            &[]
        } else {
            slice::from_raw_parts(self.ptr, self.len)
        }
    }
}

// ── PackageMetaHandle ─────────────────────────────────────────────────────────
pub type PackageMetaHandle = *mut c_void;
pub type CommitHandle = *mut c_void;

// ── CPackageEntry ─────────────────────────────────────────────────────────────
// One item in CMutatedRequest.packages (install path).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CPackageEntry {
    struct_size: usize,
    pub meta: PackageMetaHandle,
    pub temp_path: CSlice,
    pub checksum: CSlice,
}

impl CPackageEntry {
    pub fn new(meta: PackageMetaHandle, temp_path: CSlice, checksum: CSlice) -> Self {
        Self {
            struct_size: size_of::<Self>(),
            meta,
            temp_path,
            checksum,
        }
    }
}

// ── Progress callback ─────────────────────────────────────────────────────────
// event is u32 — matches Zig's CMutatedRequest.on_progress declaration.
pub type CProgressFn = unsafe extern "C" fn(event: u32, package_name: CSlice, ctx: *mut c_void);

// ── CRepoMode ─────────────────────────────────────────────────────────────────
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum CRepoMode {
    Archive = 0,
    Bare = 1,
    BareUser = 2,
}

// ── CancelToken ───────────────────────────────────────────────────────────────
// Mirrors Zig's CancelToken extern struct (24 bytes).
// _flag: u8, _hook: ?fn ptr, _hook_ctx: ?*anyopaque
#[repr(C)]
pub struct CancelToken {
    _flag: u8,
    _hook: Option<unsafe extern "C" fn(*mut c_void)>,
    _hook_ctx: *mut c_void,
}

impl CancelToken {
    pub const fn new() -> Self {
        Self {
            _flag: 0,
            _hook: None,
            _hook_ctx: null_mut(),
        }
    }

    pub fn cancel(&self) {
        unsafe {
            let atomic_flag = &*(&self._flag as *const u8 as *const AtomicU8);
            atomic_flag.store(1, Ordering::Release);
            if let Some(hook) = self._hook {
                hook(self._hook_ctx);
            }
        }
    }
}

impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}

// unsafe impl Send for CancelToken {}
// unsafe impl Sync for CancelToken {}

// ── CMutatedRequest ───────────────────────────────────────────────────────────
// Unified repr(C) struct for install, uninstall and rollback.
// CSlice fields borrow from CStrings stored in the calling machine.
#[repr(C)]
pub struct CMutatedRequest {
    struct_size: usize,

    repo_path: CSlice,
    root_path: CSlice,
    branch: CSlice,

    // install
    packages: *const CPackageEntry,
    packages_count: usize,

    // uninstall
    package_names: *const CSlice,
    package_names_len: usize,

    // rollback
    commit_hash: CSlice,

    on_progress: Option<CProgressFn>,
    progress_ctx: *mut c_void,

    max_retries: u8,
    cancel_token: *mut CancelToken,
}

impl CMutatedRequest {
    fn base(
        repo_path: &CString,
        root_path: &CString,
        branch: &CString,
        max_retries: u8,
        on_progress: Option<CProgressFn>,
        progress_ctx: *mut c_void,
        cancel_token: *mut CancelToken,
    ) -> Self {
        Self {
            struct_size: size_of::<Self>(),
            repo_path: CSlice::from_cstring(repo_path),
            root_path: CSlice::from_cstring(root_path),
            branch: CSlice::from_cstring(branch),
            packages: null(),
            packages_count: 0,
            package_names: null(),
            package_names_len: 0,
            commit_hash: CSlice::empty(),
            on_progress,
            progress_ctx,
            max_retries,
            cancel_token,
        }
    }

    /// Install.  `packages` slice must outlive this request.
    #[allow(clippy::too_many_arguments)]
    pub fn for_install(
        packages: &[CPackageEntry],
        repo_path: &CString,
        root_path: &CString,
        branch: &CString,
        max_retries: u8,
        on_progress: Option<CProgressFn>,
        progress_ctx: *mut c_void,
        cancel_token: *mut CancelToken,
    ) -> Self {
        let mut req = Self::base(repo_path, root_path, branch, max_retries, on_progress, progress_ctx, cancel_token);
        req.packages = if packages.is_empty() { null() } else { packages.as_ptr() };
        req.packages_count = packages.len();
        req
    }

    /// Uninstall.  `package_names` slice of CSlices must outlive this request.
    #[allow(clippy::too_many_arguments)]
    pub fn for_uninstall(
        package_names: &[CSlice],
        repo_path: &CString,
        root_path: &CString,
        branch: &CString,
        max_retries: u8,
        on_progress: Option<CProgressFn>,
        progress_ctx: *mut c_void,
        cancel_token: *mut CancelToken,
    ) -> Self {
        let mut req = Self::base(repo_path, root_path, branch, max_retries, on_progress, progress_ctx, cancel_token);
        req.package_names = if package_names.is_empty() { null() } else { package_names.as_ptr() };
        req.package_names_len = package_names.len();
        req
    }

    /// Rollback.
    #[allow(clippy::too_many_arguments)]
    pub fn for_rollback(
        commit_hash: &CString,
        repo_path: &CString,
        root_path: &CString,
        branch: &CString,
        max_retries: u8,
        cancel_token: *mut CancelToken,
    ) -> Self {
        let mut req = Self::base(repo_path, root_path, branch, max_retries, None, null_mut(), cancel_token);
        req.commit_hash = CSlice::from_cstring(commit_hash);
        req
    }
}

// ── CUnmutatedRequest ─────────────────────────────────────────────────────────
// Unified repr(C) struct for init, diff and list.
// `repo_mode` points to a u32 stored in the calling machine; null for diff/list.
#[repr(C)]
pub struct CUnmutatedRequest {
    struct_size: usize,

    repo_path: CSlice,
    root_path: CSlice,
    branch: CSlice,

    from_commit_hash: CSlice,
    to_commit_hash: CSlice,

    // init only: pointer into a caller-owned &[CSlice]
    symlinks: *const CSlice,
    symlinks_len: usize,

    // Points to a machine-owned u32; Zig reads it as *const i32 via @ptrCast.
    // Null is accepted for diff and list.
    repo_mode: *mut c_void,
    cancel_token: *mut CancelToken,
}

impl CUnmutatedRequest {
    fn base(
        repo_path: &CString,
        root_path: &CString,
        branch: &CString,
        cancel_token: *mut CancelToken,
    ) -> Self {
        Self {
            struct_size: size_of::<Self>(),
            repo_path: CSlice::from_cstring(repo_path),
            root_path: CSlice::from_cstring(root_path),
            branch: CSlice::from_cstring(branch),
            from_commit_hash: CSlice::empty(),
            to_commit_hash: CSlice::empty(),
            symlinks: null(),
            symlinks_len: 0,
            repo_mode: null_mut(),
            cancel_token,
        }
    }

    /// Init.  `symlinks` slice and `repo_mode_val` must outlive this request.
    pub fn for_init(
        repo_path: &CString,
        root_path: &CString,
        branch: &CString,
        symlinks: &[CSlice],
        repo_mode_val: &u32,
        cancel_token: *mut CancelToken,
    ) -> Self {
        let mut req = Self::base(repo_path, root_path, branch, cancel_token);
        req.symlinks = if symlinks.is_empty() { null() } else { symlinks.as_ptr() };
        req.symlinks_len = symlinks.len();
        req.repo_mode = repo_mode_val as *const u32 as *mut c_void;
        req
    }

    /// Diff packages or diff files attributed.
    pub fn for_diff(
        repo_path: &CString,
        root_path: &CString,
        branch: &CString,
        from_commit: &CString,
        to_commit: &CString,
        cancel_token: *mut CancelToken,
    ) -> Self {
        let mut req = Self::base(repo_path, root_path, branch, cancel_token);
        req.from_commit_hash = CSlice::from_cstring(from_commit);
        req.to_commit_hash = CSlice::from_cstring(to_commit);
        req
    }

    /// List packages.
    pub fn for_list(
        repo_path: &CString,
        root_path: &CString,
        branch: &CString,
        cancel_token: *mut CancelToken,
    ) -> Self {
        Self::base(repo_path, root_path, branch, cancel_token)
    }
}

// ── CPrepareRequest ───────────────────────────────────────────────────────────
// Passed to the backend .so `prepare` function.
// CSlice fields borrow from CStrings in the install machine.
// Note: backend progress uses u8 event (backend-specific enum).
#[repr(C)]
pub struct CPrepareRequest {
    struct_size: usize,
    checksum: CSlice,
    package_path: CSlice,
    temp_dir_path: CSlice,
    on_progress: unsafe extern "C" fn(u8, CSlice, *mut c_void),
    progress_ctx: *mut c_void,
    cancel_token: *mut CancelToken,
}

impl CPrepareRequest {
    pub fn new(
        package_path: &CString,
        temp_dir_path: &CString,
        checksum: &CString,
        on_progress: unsafe extern "C" fn(u8, CSlice, *mut c_void),
        progress_ctx: *mut c_void,
        cancel_token: *mut CancelToken,
    ) -> Self {
        Self {
            struct_size: size_of::<Self>(),
            checksum: CSlice::from_cstring(checksum),
            package_path: CSlice::from_cstring(package_path),
            temp_dir_path: CSlice::from_cstring(temp_dir_path),
            on_progress,
            progress_ctx,
            cancel_token,
        }
    }
}

// ── Diff entry types ──────────────────────────────────────────────────────────
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CDiffKind {
    Added = 0,
    Removed = 1,
    Modified = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CPackageDiffKind {
    Added = 0,
    Removed = 1,
    Updated = 2,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CPackageDiffEntry {
    struct_size: usize,
    pub name: CSlice,
    pub kind: CPackageDiffKind,
}

impl Validate for CPackageDiffEntry {
    fn validate(&self) -> Result<()> {
        if self.struct_size != size_of::<Self>() {
            return Err(anyhow::anyhow!("CPackageDiffEntry: abi mismatch"));
        }
        self.name.validate()?;
        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CAttributedDiffEntry {
    struct_size: usize,
    pub path: CSlice,
    pub kind: CDiffKind,
    pub package_name: CSlice,
}

impl Validate for CAttributedDiffEntry {
    fn validate(&self) -> Result<()> {
        if self.struct_size != size_of::<Self>() {
            return Err(anyhow::anyhow!("CAttributedDiffEntry: abi mismatch"));
        }
        self.path.validate()?;
        Ok(())
    }
}

// ── Symbol loader ─────────────────────────────────────────────────────────────
/// Loads a symbol from a dynamic library.
///
/// # Safety
///
/// The caller must ensure that:
/// 1. A symbol with the specified name actually exists in the library.
/// 2. The type `T` matches the function signature or data type in the shared library.
/// 3. The library `lib` remains loaded in memory for the entire duration of the returned value's use.
pub unsafe fn load_symbol<T: Copy>(lib: &Library, name: &str) -> Result<T> {
    lib.get(name.as_bytes())
        .map(|symbol| *symbol)
        .map_err(|err| anyhow::anyhow!("Symbol {name} not found: {err}"))
}
