// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::CString;
use std::mem::MaybeUninit;
use std::ptr::{null, null_mut};

use anyhow::Result;

use upac_abi::error::CError;
use upac_abi::package::CPackageInfo;
use upac_abi::request::CRequestBase;
use upac_abi::types::{CBorrowed, CSlice, CVec};

use crate::cancel_token_ptr;
use crate::types::errors::LibError;

pub fn request_base() -> CRequestBase {
    CRequestBase::new(None, null_mut(), cancel_token_ptr())
}

pub fn slice_from_cstr(value: &CString) -> CSlice {
    CSlice {
        ptr: value.as_ptr().cast(),
        len: value.as_bytes().len(),
    }
}

pub fn empty_slice() -> CSlice {
    CSlice { ptr: null(), len: 0 }
}

pub fn optional_slice(value: Option<&CString>) -> CSlice {
    match value {
        Some(value) => slice_from_cstr(value),
        None => empty_slice(),
    }
}

pub fn package_info(name: &CString, arch: &CString, arch_sub: Option<&CString>) -> CPackageInfo {
    CPackageInfo::new(slice_from_cstr(name), slice_from_cstr(arch), optional_slice(arch_sub))
}

pub fn borrowed_vec<T>(items: &[T]) -> CVec<T> {
    CVec::from_borrowed(items)
}

pub fn invoke(call: impl FnOnce(*mut CError) -> i32) -> Result<()> {
    let mut error = MaybeUninit::uninit();
    let code = call(error.as_mut_ptr());
    unsafe { LibError::check(code, error.as_ptr())? };
    Ok(())
}

pub fn invoke_with_response<R>(call: impl FnOnce(*mut R, *mut CError) -> i32) -> Result<R> {
    let mut response = MaybeUninit::zeroed();
    let mut error = MaybeUninit::uninit();
    let code = call(response.as_mut_ptr(), error.as_mut_ptr());
    unsafe { LibError::check(code, error.as_ptr())? };
    Ok(unsafe { response.assume_init() })
}
