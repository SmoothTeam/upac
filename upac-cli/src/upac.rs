// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::{bail, Result};

use libloading::Library;

use std::str;

use crate::ffi::{
    load_symbol, CArray, CAttributedDiffEntry, CMutatedRequest, CPackageDiffEntry, CSlice,
    CUnmutatedRequest, CancelToken, CommitHandle, PackageMetaHandle,
};
use crate::utils::BackendKind;

const EXPECTED_ABI_VERSION: u32 = 1;

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
    pub cancel: unsafe extern "C" fn(*mut CancelToken),

    _lib: Library,
}

impl UpacLib {
    // Loads the library from a file and initializes pointers to symbols
    pub fn load(backend_kind: &BackendKind) -> Result<Self> {
        let loaded_library = unsafe { Library::new(backend_kind.so_name()) }.map_err(|error| {
            anyhow::anyhow!("Failed to load {}: {error}", backend_kind.so_name())
        })?;

        let get_abi_version: unsafe extern "C" fn() -> u32 =
            unsafe { load_symbol(&loaded_library, "get_abi_version")? };
        let abi_version = unsafe { get_abi_version() };
        if abi_version != EXPECTED_ABI_VERSION {
            bail!(
                "{}: ABI version mismatch (library={abi_version}, expected={EXPECTED_ABI_VERSION})",
                backend_kind.so_name()
            );
        }

        let lib = Self {
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
            cancel: unsafe { load_symbol(&loaded_library, "cancel")? },

            _lib: loaded_library,
        };

        Ok(lib)
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
            7 => "abi mismatch",

            9 => "thread error",
            10 => "lock would block — another process is running",
            11 => "allocz failed",
            12 => return Err(anyhow::anyhow!("{context}: cancelled (code {code})")),
            13 => "max retries exceeded",
            14 => "read failed",
            15 => "write failed",
            16 => "diff failed",
            17 => "list failed",

            30 => "missing field",
            31 => "missing section",
            32 => "invalid entry",
            33 => "parse error",
            34 => "write database failed",
            35 => "malformed meta",
            36 => "malformed files",
            37 => "idx malformed entry",

            50 => "package already installed",
            51 => "package temp path not found",
            52 => "checksum calculation failed",
            53 => "checkout failed",
            54 => "install cancelled",
            55 => "max retries exceeded",
            56 => "check space failed",
            57 => "make failed",

            70 => "package not found for uninstall",
            71 => "uninstall failed",
            72 => "file map corrupted",
            73 => "staging not cleaned",

            90 => "failed to open repository",
            91 => "transaction failed",
            92 => "commit failed",
            93 => "rollback failed",
            94 => "no previous commit",
            95 => "staging checkout failed",
            96 => "atomic swap failed (renameat2)",
            97 => "commit not found",
            98 => "cleanup failed",
            99 => "repo write failed",
            100 => "mtree insert failed",

            110 => "already initialized",
            111 => "failed to create directory",
            112 => "not a directory",
            113 => "init failed",
            114 => "directory not empty",
            115 => "init prefix not found",
            116 => "init additional prefix not found",

            120 => "file checksum failed",
            121 => "file already exists",

            _ => "unknown error",
        };
        bail!("{context}: {message} (code {code})");
    }
}

