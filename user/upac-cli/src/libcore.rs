// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;

use libloading::Library;

use upac_abi::error::CError;
use upac_abi::hook::CancelToken;
use upac_abi::request::CListPackagesRequest;
use upac_abi::response::CListPackagesResponse;

use crate::types::errors::{AbiMismatch, LibError};

// ── Wrapper around libupac.so ────────────────────────────────────────────────
pub struct Lib {
    pub list_packages: unsafe extern "C" fn(CListPackagesRequest, *mut CListPackagesResponse, *mut CError) -> i32,

    pub cancel: unsafe extern "C" fn(*mut CancelToken),
    pub version_abi: unsafe extern "C" fn() -> u32,
    _lib: Library,
}

impl Lib {
    pub fn load() -> Result<Self> {
        let loaded_library = unsafe { Library::new("libupac.so") }?;

        let lib = Self {
            list_packages: unsafe { Self::load_symbol(&loaded_library, "list_packages")? },

            cancel: unsafe { Self::load_symbol(&loaded_library, "cancel")? },
            version_abi: unsafe { Self::load_symbol(&loaded_library, "version_abi")? },

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
