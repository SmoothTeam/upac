// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;

use libloading::Library;

use crate::ffi::request::{CMutatedRequest, CUnmutatedRequest, CUnmutatedResponse};
use crate::ffi::{load_symbol, CancelToken};

use crate::types::errors::AbiMismatch;
use crate::types::EXPECTED_ABI_VERSION;

pub mod backend;
pub mod registry;

// ── Wrapper around libupac.so ────────────────────────────────────────────────
pub struct LibPkg {
    pub install: unsafe extern "C" fn(CMutatedRequest) -> i32,
    pub uninstall: unsafe extern "C" fn(CMutatedRequest) -> i32,
    pub update: unsafe extern "C" fn(CMutatedRequest) -> i32,
    pub list: unsafe extern "C" fn(CUnmutatedRequest, *mut CUnmutatedResponse) -> i32,
    pub diff: unsafe extern "C" fn(CUnmutatedRequest, *mut CUnmutatedResponse) -> i32,
    pub search: unsafe extern "C" fn(CUnmutatedRequest, *mut CUnmutatedResponse) -> i32,
}

impl LibPkg {
    fn load(lib: &Library) -> Result<Self> {
        Ok(Self {
            install: unsafe { load_symbol(lib, "install")? },
            uninstall: unsafe { load_symbol(lib, "uninstall")? },
            update: unsafe { load_symbol(lib, "update")? },
            list: unsafe { load_symbol(lib, "list_metas")? },
            diff: unsafe { load_symbol(lib, "diff_packages")? },
            search: unsafe { load_symbol(lib, "search_meta")? },
        })
    }
}

pub struct LibFile {
    pub files: unsafe extern "C" fn(CMutatedRequest) -> i32,
    pub diff: unsafe extern "C" fn(CUnmutatedRequest, *mut CUnmutatedResponse) -> i32,
    pub search: unsafe extern "C" fn(CUnmutatedRequest, *mut CUnmutatedResponse) -> i32,
}

impl LibFile {
    fn load(lib: &Library) -> Result<Self> {
        Ok(Self {
            files: unsafe { load_symbol(lib, "files")? },
            diff: unsafe { load_symbol(lib, "diff_files")? },
            search: unsafe { load_symbol(lib, "search_files")? },
        })
    }
}

pub struct LibCommit {
    pub new: unsafe extern "C" fn(CMutatedRequest) -> i32,
    pub rollback: unsafe extern "C" fn(CMutatedRequest) -> i32,
    pub list: unsafe extern "C" fn(CUnmutatedRequest, *mut CUnmutatedResponse) -> i32,
}

impl LibCommit {
    fn load(lib: &Library) -> Result<Self> {
        Ok(Self {
            new: unsafe { load_symbol(lib, "commit")? },
            rollback: unsafe { load_symbol(lib, "rollback")? },
            list: unsafe { load_symbol(lib, "list_commits")? },
        })
    }
}

pub struct Lib {
    pub pkg: LibPkg,
    pub file: LibFile,
    pub commit: LibCommit,
    pub init: unsafe extern "C" fn(CUnmutatedRequest) -> i32,
    pub cancel: unsafe extern "C" fn(*mut CancelToken),
    pub free_response: unsafe extern "C" fn(*mut CUnmutatedResponse),
    pub version_abi: unsafe extern "C" fn() -> u32,
    _lib: Library,
}

impl Lib {
    pub fn load() -> Result<Self> {
        let loaded_library = unsafe { Library::new("libupac.so") }?;

        let lib = Self {
            pkg: LibPkg::load(&loaded_library)?,
            file: LibFile::load(&loaded_library)?,
            commit: LibCommit::load(&loaded_library)?,
            init: unsafe { load_symbol(&loaded_library, "init")? },

            cancel: unsafe { load_symbol(&loaded_library, "cancel")? },
            free_response: unsafe { load_symbol(&loaded_library, "free_response")? },
            version_abi: unsafe { load_symbol(&loaded_library, "version_abi")? },

            _lib: loaded_library,
        };

        let abi_version = unsafe { (lib.version_abi)() };
        if abi_version != EXPECTED_ABI_VERSION {
            let err = AbiMismatch {
                got: abi_version,
                expected: EXPECTED_ABI_VERSION,
            };

            return Err(err.into());
        }

        Ok(lib)
    }
}
