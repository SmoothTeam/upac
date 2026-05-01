// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::{bail, Result};

use libloading::Library;

use std::str;

use crate::ffi::{
    load_symbol, CArray, CAttributedDiffEntry, CMutatedRequest, CPackageDiffEntry, CSlice,
    CUnmutatedRequest, CommitHandle, PackageMetaHandle,
};
use crate::utils::BackendKind;

// ── Wrapper around libupac.so ────────────────────────────────────────────────────
// A wrapper for dynamically loading libupac.so and mapping its C functions to Rust types
pub struct UpacLib {
    pub install: unsafe extern "C" fn(CMutatedRequest) -> i32,
    pub uninstall: unsafe extern "C" fn(CMutatedRequest) -> i32,
    pub rollback: unsafe extern "C" fn(CMutatedRequest) -> i32,

    pub diff_packages:
        unsafe extern "C" fn(CUnmutatedRequest, *mut CArray<CPackageDiffEntry>) -> i32,
    pub diff_packages_free: unsafe extern "C" fn(*mut CArray<CPackageDiffEntry>),
    pub diff_files:
        unsafe extern "C" fn(CUnmutatedRequest, *mut CArray<CAttributedDiffEntry>) -> i32,
    pub diff_files_free: unsafe extern "C" fn(*mut CArray<CAttributedDiffEntry>),

    pub list_packages:
        unsafe extern "C" fn(CUnmutatedRequest, *mut CArray<PackageMetaHandle>) -> i32,
    pub get_packages_count: unsafe extern "C" fn(*mut CArray<PackageMetaHandle>) -> usize,
    pub get_package_at:
        unsafe extern "C" fn(*mut CArray<PackageMetaHandle>, u8, *mut PackageMetaHandle) -> i32,
    pub get_package_slice_field: unsafe extern "C" fn(PackageMetaHandle, u8, *mut CSlice) -> i32,
    pub get_package_int_field: unsafe extern "C" fn(PackageMetaHandle, u8, *mut u64) -> i32,
    pub packages_free: unsafe extern "C" fn(*mut CArray<PackageMetaHandle>),

    pub list_commits: unsafe extern "C" fn(CUnmutatedRequest, *mut CArray<CommitHandle>) -> i32,
    pub get_commits_count: unsafe extern "C" fn(*mut CArray<CommitHandle>) -> usize,
    pub get_commit_at:
        unsafe extern "C" fn(*mut CArray<CommitHandle>, u8, *mut CommitHandle) -> i32,
    pub get_commit_slice_field: unsafe extern "C" fn(CommitHandle, u8, *mut CSlice) -> i32,
    pub commits_free: unsafe extern "C" fn(*mut CArray<CommitHandle>),

    pub init: unsafe extern "C" fn(CUnmutatedRequest) -> i32,

    pub deinit: unsafe extern "C" fn(),

    _lib: Library,
}

impl UpacLib {
    // Loads the library from a file and initializes pointers to symbols
    pub fn load(backend_kind: &BackendKind) -> Result<Self> {
        let loaded_library = unsafe { Library::new(backend_kind.so_name()) }.map_err(|error| {
            anyhow::anyhow!("Failed to load {}: {error}", backend_kind.so_name())
        })?;

        Ok(Self {
            install: unsafe { load_symbol(&loaded_library, "install")? },
            uninstall: unsafe { load_symbol(&loaded_library, "uninstall")? },
            rollback: unsafe { load_symbol(&loaded_library, "rollback")? },

            diff_packages: unsafe { load_symbol(&loaded_library, "diff_packages")? },
            diff_packages_free: unsafe { load_symbol(&loaded_library, "diff_packages_free")? },
            diff_files: unsafe { load_symbol(&loaded_library, "diff_files")? },
            diff_files_free: unsafe { load_symbol(&loaded_library, "diff_files_free")? },

            list_packages: unsafe { load_symbol(&loaded_library, "list_packages")? },
            get_packages_count: unsafe { load_symbol(&loaded_library, "get_packages_count")? },
            get_package_at: unsafe { load_symbol(&loaded_library, "get_package_at")? },
            get_package_slice_field: unsafe {
                load_symbol(&loaded_library, "get_package_slice_field")?
            },
            get_package_int_field: unsafe {
                load_symbol(&loaded_library, "get_package_int_field")?
            },

            list_commits: unsafe { load_symbol(&loaded_library, "list_commits")? },
            get_commits_count: unsafe { load_symbol(&loaded_library, "get_commits_count")? },
            get_commit_at: unsafe { load_symbol(&loaded_library, "get_commit_at")? },
            get_commit_slice_field: unsafe {
                load_symbol(&loaded_library, "get_commit_slice_field")?
            },
            packages_free: unsafe { load_symbol(&loaded_library, "packages_free")? },
            commits_free: unsafe { load_symbol(&loaded_library, "commits_free")? },

            init: unsafe { load_symbol(&loaded_library, "init")? },
            deinit: unsafe { load_symbol(&loaded_library, "deinit")? },

            _lib: loaded_library,
        })
    }

    // Converts numeric error codes from the C-layer into human-readable anyhow::Result values
    pub fn check(code: i32, context: &str) -> Result<()> {
        let message = match code {
            0 => return Ok(()),

            1 => "unexpected error",
            2 => "out of memory",
            3 => "file not found",
            4 => "permission_denied",
            5 => "invalid path",
            6 => "no space left",
            7 => "abi_mismatch",

            9 => "thread error",
            10 => "lock would block — another process is running",
            11 => "allocz_failed",
            12 => return Err(anyhow::anyhow!("{context}: cancelled (code {code})")),
            13 => "max_retries_exceeded",
            14 => "read_failed",
            15 => "write_failed",

            20 => "database: missing field",
            21 => "database: missing section",
            22 => "database: invalid entry",
            23 => "database: parse error",
            24 => "database: write database failed",
            25 => "db_malformed_meta",
            26 => "db_malformed_files",
            27 => "idx_malformed_entry",

            30 => "package already installed",
            31 => "install: package temp path not found",
            32 => "install: checksum calculation failed",
            33 => "install: checkout failed",
            34 => "install: install cancelled",
            35 => "install: max retries exceeded",
            36 => "install: check space failed",
            37 => "install: make failed",

            40 => "package not found for uninstall",
            41 => "uninstall failed",
            42 => "uninstall: file map corrupted",
            43 => "uninstall: staging not cleaned",

            50 => "ostree: failed to open repository",
            51 => "ostree: transaction failed",
            52 => "ostree: commit failed",
            53 => "ostree: diff failed",
            54 => "ostree: rollback failed",
            55 => "ostree: no previous commit",
            56 => "ostree: staging checkout failed",
            57 => "ostree: atomic swap failed (renameat2)",
            58 => "ostree: commit not found",
            59 => "ostree: cleanup failed",
            65 => "ostree: repo write failed",
            66 => "ostree: mtree insert failed",

            60 => "already initialized",
            61 => "failed to create directory",
            62 => "ostree: init failed",
            63 => "ostree: init failed",
            64 => "directory not empty",
            67 => "init prefix not found",
            68 => "init additional prefix not found",

            70 => "file checksum failed",
            71 => "file already exists",

            _ => "unknown error",
        };
        bail!("{context}: {message} (code {code})");
    }
}

impl Drop for UpacLib {
    fn drop(&mut self) {
        unsafe { (self.deinit)() }
    }
}
