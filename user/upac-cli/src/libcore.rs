// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;

use libloading::Library;

use nix::unistd::Uid;

use upac_abi::error::CError;
use upac_abi::hook::CancelToken;
use upac_abi::request::{
    CCommitRequest, CDiffPackagesRequest, CDiffPrefixRequest, CFilesRequest, CGcRequest, CInstallRequest,
    CListConfigRequest, CListHistoryRequest, CListPackagesRequest, CRollbackRequest, CSearchFilesRequest,
    CSearchMetaRequest, CUninstallRequest, CUpdateRequest,
};
use upac_abi::response::{
    CDiffPackagesResponse, CDiffPrefixResponse, CListConfigResponse, CListHistoryResponse, CListPackagesResponse,
    CSearchFilesResponse, CSearchMetaResponse,
};

use crate::types::errors::{AbiMismatch, LibError};

// ── Symbol loading ────────────────────────────────────────────────────────────
pub trait LoadLibrarySymbols: Sized {
    fn load(lib: &Library) -> Result<Self>;
}

// ── Read-only symbols ─────────────────────────────────────────────────────────
pub struct RoSymbols {
    pub list_packages: unsafe extern "C" fn(CListPackagesRequest, *mut CListPackagesResponse, *mut CError) -> i32,
    pub search_meta: unsafe extern "C" fn(CSearchMetaRequest, *mut CSearchMetaResponse, *mut CError) -> i32,
    pub diff_packages: unsafe extern "C" fn(CDiffPackagesRequest, *mut CDiffPackagesResponse, *mut CError) -> i32,
    pub list_config: unsafe extern "C" fn(CListConfigRequest, *mut CListConfigResponse, *mut CError) -> i32,
    pub list_history: unsafe extern "C" fn(CListHistoryRequest, *mut CListHistoryResponse, *mut CError) -> i32,
    pub diff_prefix: unsafe extern "C" fn(CDiffPrefixRequest, *mut CDiffPrefixResponse, *mut CError) -> i32,
    pub search_files: unsafe extern "C" fn(CSearchFilesRequest, *mut CSearchFilesResponse, *mut CError) -> i32,
}

impl LoadLibrarySymbols for RoSymbols {
    fn load(lib: &Library) -> Result<Self> {
        Ok(Self {
            list_packages: unsafe { load_symbol(lib, "list_packages")? },
            search_meta: unsafe { load_symbol(lib, "search_meta")? },
            diff_packages: unsafe { load_symbol(lib, "diff_packages")? },
            list_config: unsafe { load_symbol(lib, "list_config")? },
            list_history: unsafe { load_symbol(lib, "list_history")? },
            diff_prefix: unsafe { load_symbol(lib, "diff_prefix")? },
            search_files: unsafe { load_symbol(lib, "search_files")? },
        })
    }
}

// ── Mutating symbols ──────────────────────────────────────────────────────────
pub struct RwSymbols {
    pub install: unsafe extern "C" fn(CInstallRequest, *mut CError) -> i32,
    pub update: unsafe extern "C" fn(CUpdateRequest, *mut CError) -> i32,
    pub uninstall: unsafe extern "C" fn(CUninstallRequest, *mut CError) -> i32,
    pub commit: unsafe extern "C" fn(CCommitRequest, *mut CError) -> i32,
    pub rollback: unsafe extern "C" fn(CRollbackRequest, *mut CError) -> i32,
    pub files: unsafe extern "C" fn(CFilesRequest, *mut CError) -> i32,
    pub gc: unsafe extern "C" fn(CGcRequest, *mut CError) -> i32,
}

impl LoadLibrarySymbols for RwSymbols {
    fn load(lib: &Library) -> Result<Self> {
        Ok(Self {
            install: unsafe { load_symbol(lib, "install")? },
            update: unsafe { load_symbol(lib, "update")? },
            uninstall: unsafe { load_symbol(lib, "uninstall")? },
            commit: unsafe { load_symbol(lib, "commit")? },
            rollback: unsafe { load_symbol(lib, "rollback")? },
            files: unsafe { load_symbol(lib, "files")? },
            gc: unsafe { load_symbol(lib, "gc")? },
        })
    }
}

// ── Wrapper around libupac.so ────────────────────────────────────────────────
pub struct Lib {
    pub ro: RoSymbols,
    pub rw: RwSymbols,

    pub cancel: unsafe extern "C" fn(*mut CancelToken),
    pub version_abi: unsafe extern "C" fn() -> u32,
    _lib: Library,
}

impl Lib {
    pub fn load() -> Result<Self> {
        let loaded_library = unsafe { Library::new("libupac.so") }?;

        let lib = Self {
            ro: RoSymbols::load(&loaded_library)?,
            rw: RwSymbols::load(&loaded_library)?,

            cancel: unsafe { load_symbol(&loaded_library, "cancel")? },
            version_abi: unsafe { load_symbol(&loaded_library, "version_abi")? },

            _lib: loaded_library,
        };

        let abi_version = unsafe { (lib.version_abi)() };
        if abi_version != upac_abi::ABI_VERSION {
            let err = AbiMismatch {
                got: abi_version,
                expected: upac_abi::ABI_VERSION,
            };

            return Err(err.into());
        }

        Ok(lib)
    }

    /// Gates access to the mutating symbol table behind an effective-root check — call sites for
    /// install/update/uninstall/commit/rollback/files/gc go through here instead of reading `self.rw`
    /// directly, so the check can't be forgotten at a new call site.
    pub fn require_write(&self) -> Result<&RwSymbols> {
        if !Uid::effective().is_root() {
            anyhow::bail!(gettextrs::gettext("err_requires_root"));
        }

        Ok(&self.rw)
    }
}

/// # Safety
/// `T` must exactly match the signature of the C symbol `name` resolves to, and the returned value
/// must not outlive `lib`.
unsafe fn load_symbol<T: Copy>(lib: &Library, name: &str) -> Result<T> {
    unsafe {
        lib.get(name.as_bytes())
            .map(|symbol| *symbol)
            .map_err(|err| anyhow::anyhow!("Symbol {name} not found: {err}"))
    }
}

impl LibError {
    /// # Safety
    /// `error` must point to a valid, initialized `CError` whenever `code != 0` — the ABI only writes
    /// to it on the failure path, leaving it uninitialized on success.
    pub unsafe fn check(code: i32, error: *const CError) -> Result<(), Self> {
        if code == 0 {
            return Ok(());
        }
        Err(Self {
            error: unsafe { *error },
        })
    }
}
