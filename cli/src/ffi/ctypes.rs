use anyhow::Result;

use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::ptr::{null, null_mut};
use std::slice;
use std::str;

use super::Validate;

// ── CSlice ────────────────────────────────────────────────────────────────────
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

    pub fn empty_str() -> Self {
        static NULL_BYTE: u8 = 0;
        Self {
            ptr: &raw const NULL_BYTE,
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
        str::from_utf8_unchecked(slice::from_raw_parts(self.ptr, self.len))
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
#[derive(Clone, Copy)]
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

// ── CVersion ──────────────────────────────────────────────────────────────────
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CVersion {
    struct_size: usize,

    pub epoch: u32,
    pub release: u32,
    pub parts: CArray<u32>,
    pub pre: CSlice,
}

impl CVersion {
    pub unsafe fn display(&self) -> String {
        let parts = self.parts.as_slice();
        let version_str = parts
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(".");

        let mut result = if self.epoch > 0 {
            format!("{}:{}", self.epoch, version_str)
        } else {
            version_str
        };

        if self.release > 0 {
            result.push('-');
            result.push_str(&self.release.to_string());
        }

        if !self.pre.ptr.is_null() && self.pre.len > 0 {
            result.push('~');
            result.push_str(self.pre.as_str());
        }

        result
    }
}

// ── CDiffKind ─────────────────────────────────────────────────────────────────
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CDiffKind {
    Added = 0,
    Removed = 1,
    Modified = 2,
}

// ── CRepoMode ─────────────────────────────────────────────────────────────────
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum CRepoMode {
    Archive = 0,
    Bare = 1,
    BareUser = 2,
}

// ── CHookPreInstall ───────────────────────────────────────────────────────────
#[repr(C)]
pub struct CHookPreInstall {
    pub packages_count: u32,
    pub required_space: u64,
    pub free_space: u64,
}
