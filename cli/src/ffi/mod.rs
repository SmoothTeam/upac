use anyhow::Result;
use libloading::Library;

use std::ffi::c_void;
use std::ptr::null_mut;
use std::str;
use std::sync::atomic::{AtomicU8, Ordering};

pub mod commit;
pub mod ctypes;
pub mod file;
pub mod packages;
pub mod request;

// ── Validate ──────────────────────────────────────────────────────────────────
pub trait Validate {
    fn validate(&self) -> Result<()>;
}

// ── HookFn ────────────────────────────────────────────────────────────────────
pub type HookFn = unsafe extern "C" fn(event: u32, data: *const c_void, ctx: *mut c_void) -> u8;

// ── CancelToken ───────────────────────────────────────────────────────────────
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

// ── DiffKind ──────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiffKind {
    Added,
    Removed,
    Modified,
}

// ── Symbol loader ─────────────────────────────────────────────────────────────
pub unsafe fn load_symbol<T: Copy>(lib: &Library, name: &str) -> Result<T> {
    lib.get(name.as_bytes())
        .map(|symbol| *symbol)
        .map_err(|err| anyhow::anyhow!("Symbol {name} not found: {err}"))
}
