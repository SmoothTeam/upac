use std::mem::size_of;
use std::os::raw::c_void;
use std::ptr::{null, null_mut};
use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::str::from_utf8;

unsafe extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

// ── AbiError ────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbiError {
    AbiMismatch,
    InvalidEntry,
}

// ── struct_size guard ───────────────────────────────────────────────────────
#[inline]
pub fn check_size<T>(struct_size: usize) -> Result<(), AbiError> {
    if struct_size != size_of::<T>() {
        return Err(AbiError::AbiMismatch);
    }
    Ok(())
}

// ── CSlice ──────────────────────────────────────────────────────────────────
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CSlice {
    pub ptr: *const u8,
    pub len: usize,
}

impl CSlice {
    pub unsafe fn as_slice(&self) -> &[u8] {
        if self.ptr.is_null() {
            return &[];
        }
        unsafe { from_raw_parts(self.ptr, self.len) }
    }

    pub unsafe fn as_str(&self) -> Result<&str, AbiError> {
        from_utf8(unsafe { self.as_slice() }).map_err(|_| AbiError::InvalidEntry)
    }

    pub fn from_slice(slice: Option<&[u8]>) -> CSlice {
        match slice {
            None => CSlice { ptr: null(), len: 0 },
            Some(string) => CSlice {
                ptr: string.as_ptr(),
                len: string.len(),
            },
        }
    }

    pub unsafe fn validate(&self) -> Result<(), AbiError> {
        if self.ptr.is_null() {
            return Err(AbiError::InvalidEntry);
        }
        unsafe {
            if *self.ptr.add(self.len) != 0 {
                return Err(AbiError::InvalidEntry);
            }
            let mut index = 0usize;
            while *self.ptr.add(index) != 0 {
                index += 1;
            }
            if index != self.len {
                return Err(AbiError::InvalidEntry);
            }
        }
        Ok(())
    }
}

pub unsafe fn free_cslice(string: &CSlice) {
    if string.ptr.is_null() || string.len == 0 {
        return;
    }
    unsafe { free(string.ptr as *mut c_void) };
}

// ── CArray<T> ───────────────────────────────────────────────────────────────
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CArray<T> {
    pub ptr: *mut T,
    pub len: usize,
}

impl<T> CArray<T> {
    pub unsafe fn as_slice(&self) -> &[T] {
        if self.len == 0 {
            return &[];
        }
        unsafe { from_raw_parts(self.ptr, self.len) }
    }

    pub unsafe fn as_mut_slice(&mut self) -> &mut [T] {
        if self.len == 0 {
            return &mut [];
        }
        unsafe { from_raw_parts_mut(self.ptr, self.len) }
    }
}

pub unsafe fn free_carray<T>(array: &CArray<T>) {
    if array.ptr.is_null() || array.len == 0 {
        return;
    }
    unsafe { free(array.ptr as *mut c_void) };
}

pub unsafe fn free_carray_owning<T>(array: &CArray<T>, mut free_elem: impl FnMut(&T)) {
    if array.ptr.is_null() || array.len == 0 {
        return;
    }
    unsafe {
        let slice = from_raw_parts(array.ptr, array.len);
        for element in slice {
            free_elem(element);
        }
        free(array.ptr as *mut c_void);
    }
}

pub unsafe fn alloc_bytes(count: usize) -> *mut u8 {
    if count == 0 {
        return null_mut();
    }
    unsafe { malloc(count) as *mut u8 }
}
