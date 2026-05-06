// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;
use indicatif::ProgressBar;

use strum::FromRepr;

use std::ffi::{c_void, CString};
use std::mem::MaybeUninit;
use std::ptr::null_mut;

use libloading::Library;

use crate::ffi::{load_symbol, CPrepareRequest, CSlice, CancelToken, PackageMetaHandle};
use crate::utils::BackendKind;

// ── Backend Definition ────────────────────────────────────────
#[derive(FromRepr)]
#[repr(u8)]
pub enum BackendEvent {
    Verifying = 0,
    Extracting = 1,
    ReadingMeta = 2,
    Status = 3,
    Ready = 4,
    Failed = 5,
}

impl BackendEvent {
    pub fn format_message(&self, detail_string: &str) -> String {
        match self {
            Self::Verifying => format!("Verifying {detail_string}..."),
            Self::Extracting => format!("Extracting {detail_string}..."),
            Self::ReadingMeta => "Reading metadata...".to_string(),
            Self::Status => detail_string.to_string(),
            Self::Ready => "Ready".to_string(),
            Self::Failed => "Failed".to_string(),
        }
    }
}

// ── Wrapper for the backend .so ───────────────────────────────────────────────
pub struct Backend {
    _lib: Library,

    pub prepare:
        unsafe extern "C" fn(*const CPrepareRequest, *mut PackageMetaHandle, *mut CSlice) -> i32,
    pub meta_free: unsafe extern "C" fn(PackageMetaHandle),
    pub meta_get_name: unsafe extern "C" fn(PackageMetaHandle) -> CSlice,
    pub meta_get_version: unsafe extern "C" fn(PackageMetaHandle) -> CSlice,
    pub cleanup: unsafe extern "C" fn(CSlice),
}

impl Backend {
    pub fn load(backend_kind: &BackendKind) -> Result<Self> {
        let loaded_library = unsafe { Library::new(backend_kind.so_name()) }.map_err(|error| {
            anyhow::anyhow!("Failed to load {}: {error}", backend_kind.so_name())
        })?;

        Ok(Self {
            prepare: unsafe { load_symbol(&loaded_library, "prepare")? },
            meta_free: unsafe { load_symbol(&loaded_library, "meta_free")? },
            meta_get_name: unsafe { load_symbol(&loaded_library, "meta_get_name")? },
            meta_get_version: unsafe { load_symbol(&loaded_library, "meta_get_version")? },
            cleanup: unsafe { load_symbol(&loaded_library, "cleanup")? },
            _lib: loaded_library,
        })
    }

    /// Prepare a package. All string arguments are owned CStrings stored in
    /// the calling machine and must outlive this call.
    pub fn meta_prepare(
        &self,
        pkg_path: &CString,
        temp_dir: &CString,
        checksum: &CString,
        progress_ctx: *mut c_void,
        cancel_token: *mut CancelToken,
    ) -> Result<(PackageMetaHandle, CSlice)> {
        let prepare_request_c = CPrepareRequest::new(
            pkg_path,
            temp_dir,
            checksum,
            Backend::on_backend_progress,
            progress_ctx,
            cancel_token,
        );

        let mut meta_handle: PackageMetaHandle = null_mut();
        let mut temp_path = MaybeUninit::<CSlice>::uninit();

        match unsafe {
            (self.prepare)(&prepare_request_c, &mut meta_handle, temp_path.as_mut_ptr())
        } {
            0 if !meta_handle.is_null() => Ok((meta_handle, unsafe { temp_path.assume_init() })),
            0 => anyhow::bail!("Backend returned success code but NULL handle"),
            code => anyhow::bail!("Backend prepare failed with code {code}"),
        }
    }

    pub unsafe extern "C" fn on_backend_progress(
        event_code: u8,
        detail_c_slice: CSlice,
        progress_context: *mut c_void,
    ) {
        let Some(progress_bar) = (progress_context as *const ProgressBar).as_ref() else {
            return;
        };

        let detail_string = detail_c_slice.as_str();

        let message_string = match BackendEvent::from_repr(event_code) {
            Some(event) => event.format_message(detail_string),
            None => format!("Unknown backend event code {event_code}"),
        };

        progress_bar.set_message(message_string);
    }
}
