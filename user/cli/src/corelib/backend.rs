// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;
use indicatif::ProgressBar;

use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::ptr::null_mut;

use libloading::Library;

use crate::ffi::ctypes::CSlice;
use crate::ffi::load_symbol;
use crate::ffi::packages::CPackageMeta;
use crate::ffi::request::CPrepareRequest;

use crate::types::backend::BackendEvent;
use crate::types::errors::{AbiMismatch, PrepareError};
use crate::types::EXPECTED_ABI_VERSION;

// ── Wrapper for the backend .so ───────────────────────────────────────────────
pub struct Backend {
    pub prepare: unsafe extern "C" fn(*const CPrepareRequest, *mut *mut CPackageMeta, *mut CSlice) -> i32,

    pub free_meta: unsafe extern "C" fn(*mut CPackageMeta),
    pub cleanup: unsafe extern "C" fn(CSlice),

    pub version_abi: unsafe extern "C" fn() -> u32,

    _lib: Library,
}

impl Backend {
    /// Resolution is delegated to the dynamic linker (LD_LIBRARY_PATH, /etc/ld.so.conf, rpath).
    pub fn load(so_name: &str) -> Result<Self> {
        let loaded_library = unsafe { Library::new(so_name) }?;

        let backend = Self {
            prepare: unsafe { load_symbol(&loaded_library, "prepare")? },

            free_meta: unsafe { load_symbol(&loaded_library, "free_meta")? },
            cleanup: unsafe { load_symbol(&loaded_library, "cleanup")? },
            version_abi: unsafe { load_symbol(&loaded_library, "version_abi")? },

            _lib: loaded_library,
        };

        let abi_version = unsafe { (backend.version_abi)() };
        if abi_version != EXPECTED_ABI_VERSION {
            let err = AbiMismatch {
                got: abi_version,
                expected: EXPECTED_ABI_VERSION,
            };

            return Err(err.into());
        }

        Ok(backend)
    }

    pub fn prepare(&self, request: &CPrepareRequest) -> Result<(*mut CPackageMeta, CSlice)> {
        let mut meta_ptr: *mut CPackageMeta = null_mut();
        let mut temp_path = MaybeUninit::<CSlice>::uninit();

        match unsafe { (self.prepare)(request, &mut meta_ptr, temp_path.as_mut_ptr()) } {
            0 if !meta_ptr.is_null() => Ok((meta_ptr, unsafe { temp_path.assume_init() })),
            0 => Err(PrepareError::NullMeta.into()),
            code => Err(PrepareError::Failed { code }.into()),
        }
    }

    /// # Safety
    /// `ctx` must point to a live `ProgressBar` for the duration of the call (as passed to the backend
    /// when this callback was registered), and `data`, if non-null, must point to a valid `CSlice`.
    pub unsafe extern "C" fn on_hook(event_code: u32, data: *const c_void, ctx: *mut c_void) -> u8 {
        let Some(progress_bar) = (ctx as *const ProgressBar).as_ref() else {
            return 0;
        };

        let detail_string = if data.is_null() {
            ""
        } else {
            (*(data as *const CSlice)).as_str()
        };

        let Some(event) = BackendEvent::from_repr(event_code as u8) else {
            return 0;
        };
        let message_string = event.format_message(detail_string);

        progress_bar.set_message(message_string);
        0
    }
}
