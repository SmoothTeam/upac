// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::os::raw::c_void;
use std::ptr::null_mut;
use std::slice::from_raw_parts;

use crate::types::{CSlice, CVec};

unsafe extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

/// # Safety
/// The returned pointer (if non-null) holds `count` uninitialized bytes allocated via the C allocator.
/// The caller must fully initialize it before reading, and must free it through a matching helper
/// (`free_cslice`/`free_cvec`) exactly once.
pub unsafe fn alloc_bytes(count: usize) -> *mut u8 {
    if count == 0 {
        return null_mut();
    }
    unsafe { malloc(count) as *mut u8 }
}

/// # Safety
/// `string.ptr` must be null or have been allocated via `alloc_bytes`/`CSlice::from_owned`, and must not
/// be freed more than once.
pub unsafe fn free_cslice(string: &CSlice) {
    if string.ptr.is_null() || string.len == 0 {
        return;
    }
    unsafe { free(string.ptr as *mut c_void) };
}

/// # Safety
/// `array.ptr` must be null or point to `array.len` elements of `T` allocated via `alloc_bytes`/`CVec::from_owned`,
/// and must not be freed more than once.
pub unsafe fn free_cvec<T>(array: &CVec<T>) {
    if array.ptr.is_null() || array.len == 0 {
        return;
    }
    unsafe { free(array.ptr as *mut c_void) };
}

/// # Safety
/// Same contract as `free_cvec`. `free_elem` must fully release any resources owned by each element
/// before the backing buffer itself is freed.
pub unsafe fn free_cvec_owning<T>(array: &CVec<T>, mut free_elem: impl FnMut(&T)) {
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
