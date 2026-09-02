// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::CString;

use upac_abi::error::{CError, ErrorDomain, ErrorKind};
use upac_abi::types::CSlice;

use crate::locale;
use crate::types::abi::{
    borrowed_vec, empty_slice, invoke, invoke_with_response, optional_slice, package_info, request_base,
    slice_from_cstr,
};

fn as_str(slice: &CSlice) -> &str {
    <&str>::try_from(slice).unwrap()
}

#[test]
fn slice_from_cstr_preserves_the_bytes() {
    let value = CString::new("hello").unwrap();

    assert_eq!(as_str(&slice_from_cstr(&value)), "hello");
}

#[test]
fn empty_slice_is_null_and_zero_length() {
    let slice = empty_slice();

    assert!(slice.ptr.is_null());
    assert_eq!(slice.len, 0);
}

#[test]
fn optional_slice_some_preserves_the_bytes() {
    let value = CString::new("hello").unwrap();

    assert_eq!(as_str(&optional_slice(Some(&value))), "hello");
}

#[test]
fn optional_slice_none_is_empty() {
    let slice = optional_slice(None);

    assert!(slice.ptr.is_null());
    assert_eq!(slice.len, 0);
}

#[test]
fn package_info_builds_the_expected_fields() {
    let name = CString::new("upac").unwrap();
    let arch = CString::new("x86_64").unwrap();
    let arch_sub = CString::new("v3").unwrap();

    let info = package_info(&name, &arch, Some(&arch_sub));

    assert_eq!(as_str(&info.name), "upac");
    assert_eq!(as_str(&info.arch), "x86_64");
    assert_eq!(as_str(&info.arch_sub), "v3");
}

#[test]
fn package_info_without_arch_sub_leaves_it_empty() {
    let name = CString::new("upac").unwrap();
    let arch = CString::new("x86_64").unwrap();

    let info = package_info(&name, &arch, None);

    assert!(info.arch_sub.ptr.is_null());
}

#[test]
fn borrowed_vec_wraps_the_slice_without_copying() {
    let items = [1u32, 2, 3];

    let vec = borrowed_vec(&items);

    assert_eq!(vec.len, 3);
    assert_eq!(vec.ptr, items.as_ptr() as *mut u32);
}

#[test]
fn request_base_has_no_hook_and_a_non_null_cancel_token() {
    let base = request_base();

    assert!(base.on_hook.is_none());
    assert!(base.hook_ctx.is_null());
    assert!(!base.cancel_token.is_null());
}

#[test]
fn invoke_returns_ok_on_a_zero_code() {
    assert!(invoke(|_error| 0).is_ok());
}

#[test]
fn invoke_propagates_the_localized_error_on_a_nonzero_code() {
    locale::init_for_test();

    let result = invoke(|error| unsafe {
        *error = CError {
            domain: ErrorDomain::Install,
            state: 0,
            error: ErrorKind::NotFound,
        };
        1
    });

    assert_eq!(result.unwrap_err().to_string(), "File not found (Install: Pre-hooks)");
}

#[test]
fn invoke_with_response_returns_the_response_on_a_zero_code() {
    let result = invoke_with_response(|response: *mut u32, _error| {
        unsafe { *response = 42 };
        0
    });

    assert_eq!(result.unwrap(), 42);
}

#[test]
fn invoke_with_response_propagates_the_localized_error_on_a_nonzero_code() {
    locale::init_for_test();

    let result = invoke_with_response(|_response: *mut u32, error| unsafe {
        *error = CError {
            domain: ErrorDomain::Install,
            state: 0,
            error: ErrorKind::NotFound,
        };
        1
    });

    assert_eq!(result.unwrap_err().to_string(), "File not found (Install: Pre-hooks)");
}
