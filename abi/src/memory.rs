use std::os::raw::c_void;
use std::ptr::null_mut;
use std::slice::from_raw_parts;

use crate::types::{CSlice, CVec};

unsafe extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

pub unsafe fn alloc_bytes(count: usize) -> *mut u8 {
    if count == 0 {
        return null_mut();
    }
    unsafe { malloc(count) as *mut u8 }
}

pub unsafe fn free_cslice(string: &CSlice) {
    if string.ptr.is_null() || string.len == 0 {
        return;
    }
    unsafe { free(string.ptr as *mut c_void) };
}

pub unsafe fn free_cvec<T>(array: &CVec<T>) {
    if array.ptr.is_null() || array.len == 0 {
        return;
    }
    unsafe { free(array.ptr as *mut c_void) };
}

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
