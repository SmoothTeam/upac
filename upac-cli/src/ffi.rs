// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;

use libloading::Library;

use std::ffi::{c_void, CStr, CString};
use std::marker::PhantomData;
use std::ptr::{null, null_mut};
use std::slice;
use std::str;

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

    pub unsafe fn as_str(&self) -> &str {
        let bytes = slice::from_raw_parts(self.ptr, self.len);
        str::from_utf8_unchecked(bytes)
    }
}

impl Validate for CSlice {
    fn validate(&self) -> Result<()> {
        if self.ptr.is_null() || self.len == 0 {
            return Err(anyhow::anyhow!("empty slice"));
        }
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

// ── CMutatedRequest ───────────────────────────────────────────────────────────
// Unified repr(C) struct for install, uninstall and rollback.
// CSlice fields borrow from CStrings stored in the calling machine.
#[repr(C)]
pub struct CMutatedRequest {
    struct_size: usize,

    repo_path: CSlice,
    root_path: CSlice,
    db_path: CSlice,
    branch: CSlice,
    prefix_directory: CSlice,

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
}

impl CMutatedRequest {
    fn base(
        repo_path: &CString,
        root_path: &CString,
        db_path: &CString,
        branch: &CString,
        prefix_directory: &CString,
        max_retries: u8,
        on_progress: Option<CProgressFn>,
        progress_ctx: *mut c_void,
    ) -> Self {
        Self {
            struct_size: size_of::<Self>(),
            repo_path: CSlice::from_cstring(repo_path),
            root_path: CSlice::from_cstring(root_path),
            db_path: CSlice::from_cstring(db_path),
            branch: CSlice::from_cstring(branch),
            prefix_directory: CSlice::from_cstring(prefix_directory),
            packages: null(),
            packages_count: 0,
            package_names: null(),
            package_names_len: 0,
            commit_hash: CSlice::empty(),
            on_progress,
            progress_ctx,
            max_retries,
        }
    }

    /// Install.  `packages` slice must outlive this request.
    pub fn for_install(
        packages: &[CPackageEntry],
        repo_path: &CString,
        root_path: &CString,
        db_path: &CString,
        branch: &CString,
        prefix_directory: &CString,
        max_retries: u8,
        on_progress: Option<CProgressFn>,
        progress_ctx: *mut c_void,
    ) -> Self {
        let mut req = Self::base(
            repo_path,
            root_path,
            db_path,
            branch,
            prefix_directory,
            max_retries,
            on_progress,
            progress_ctx,
        );
        req.packages = if packages.is_empty() {
            null()
        } else {
            packages.as_ptr()
        };
        req.packages_count = packages.len();
        req
    }

    /// Uninstall.  `package_names` slice of CSlices must outlive this request.
    pub fn for_uninstall(
        package_names: &[CSlice],
        repo_path: &CString,
        root_path: &CString,
        db_path: &CString,
        branch: &CString,
        prefix_directory: &CString,
        max_retries: u8,
        on_progress: Option<CProgressFn>,
        progress_ctx: *mut c_void,
    ) -> Self {
        let mut req = Self::base(
            repo_path,
            root_path,
            db_path,
            branch,
            prefix_directory,
            max_retries,
            on_progress,
            progress_ctx,
        );
        req.package_names = if package_names.is_empty() {
            null()
        } else {
            package_names.as_ptr()
        };
        req.package_names_len = package_names.len();
        req
    }

    /// Rollback.
    pub fn for_rollback(
        commit_hash: &CString,
        repo_path: &CString,
        root_path: &CString,
        db_path: &CString,
        branch: &CString,
        prefix_directory: &CString,
        max_retries: u8,
    ) -> Self {
        let mut req = Self::base(
            repo_path,
            root_path,
            db_path,
            branch,
            prefix_directory,
            max_retries,
            None,
            null_mut(),
        );
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
    db_path: CSlice,
    branch: CSlice,
    prefix: CSlice,

    from_commit_hash: CSlice,
    to_commit_hash: CSlice,

    // Points to a machine-owned u32; Zig reads it as *const i32 via @ptrCast.
    // Null is accepted for diff and list.
    repo_mode: *mut c_void,
}

impl CUnmutatedRequest {
    fn base(
        repo_path: &CString,
        root_path: &CString,
        db_path: &CString,
        branch: &CString,
        prefix: &CString,
    ) -> Self {
        Self {
            struct_size: size_of::<Self>(),
            repo_path: CSlice::from_cstring(repo_path),
            root_path: CSlice::from_cstring(root_path),
            db_path: CSlice::from_cstring(db_path),
            branch: CSlice::from_cstring(branch),
            prefix: CSlice::from_cstring(prefix),
            from_commit_hash: CSlice::empty(),
            to_commit_hash: CSlice::empty(),
            repo_mode: null_mut(),
        }
    }

    /// Init.  `repo_mode_val` must be stored in the machine and outlive the request.
    pub fn for_init(
        repo_path: &CString,
        root_path: &CString,
        db_path: &CString,
        branch: &CString,
        prefix: &CString,
        repo_mode_val: &u32,
    ) -> Self {
        let mut req = Self::base(repo_path, root_path, db_path, branch, prefix);
        req.repo_mode = repo_mode_val as *const u32 as *mut c_void;
        req
    }

    /// Diff packages or diff files attributed.
    /// `db_path` can be an empty CString for packages-only diff.
    pub fn for_diff(
        repo_path: &CString,
        root_path: &CString,
        db_path: &CString,
        branch: &CString,
        prefix: &CString,
        from_commit: &CString,
        to_commit: &CString,
    ) -> Self {
        let mut req = Self::base(repo_path, root_path, db_path, branch, prefix);
        req.from_commit_hash = CSlice::from_cstring(from_commit);
        req.to_commit_hash = CSlice::from_cstring(to_commit);
        req
    }

    /// List packages.
    pub fn for_list(
        repo_path: &CString,
        root_path: &CString,
        db_path: &CString,
        branch: &CString,
        prefix: &CString,
    ) -> Self {
        Self::base(repo_path, root_path, db_path, branch, prefix)
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
}

impl CPrepareRequest {
    pub fn new(
        package_path: &CString,
        temp_dir_path: &CString,
        checksum: &CString,
        on_progress: unsafe extern "C" fn(u8, CSlice, *mut c_void),
        progress_ctx: *mut c_void,
    ) -> Self {
        Self {
            struct_size: size_of::<Self>(),
            checksum: CSlice::from_cstring(checksum),
            package_path: CSlice::from_cstring(package_path),
            temp_dir_path: CSlice::from_cstring(temp_dir_path),
            on_progress,
            progress_ctx,
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
pub unsafe fn load_symbol<T: Copy>(lib: &Library, name: &str) -> Result<T> {
    lib.get(name.as_bytes())
        .map(|symbol| *symbol)
        .map_err(|err| anyhow::anyhow!("Symbol {name} not found: {err}"))
}
