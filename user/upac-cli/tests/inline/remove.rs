// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::CString;
use std::ptr::null;

use upac_abi::types::CSlice;

use crate::locale;

use super::{InstalledEntry, cslice_owned, find_installed, optional_cslice_owned};

fn slice_from_cstr(value: &CString) -> CSlice {
    CSlice {
        ptr: value.as_ptr().cast(),
        len: value.as_bytes().len(),
    }
}

fn null_slice() -> CSlice {
    CSlice { ptr: null(), len: 0 }
}

#[test]
fn cslice_owned_reads_the_string_content() {
    let value = CString::new("upac").unwrap();

    assert_eq!(cslice_owned(&slice_from_cstr(&value)).unwrap(), "upac");
}

#[test]
fn cslice_owned_fails_on_a_null_pointer() {
    assert!(cslice_owned(&null_slice()).is_err());
}

#[test]
fn optional_cslice_owned_is_none_for_a_null_slice() {
    assert_eq!(optional_cslice_owned(&null_slice()).unwrap(), None);
}

#[test]
fn optional_cslice_owned_is_some_for_a_populated_slice() {
    let value = CString::new("x86_64").unwrap();

    assert_eq!(
        optional_cslice_owned(&slice_from_cstr(&value)).unwrap(),
        Some("x86_64".to_owned())
    );
}

#[test]
fn find_installed_fails_when_no_package_matches_the_name() {
    locale::init_for_test();
    let installed: Vec<InstalledEntry> = Vec::new();

    let error = find_installed(&installed, "upac").unwrap_err();

    assert_eq!(error.to_string(), "Package not found: upac");
}

#[test]
fn find_installed_resolves_the_single_match_without_prompting() {
    let installed: Vec<InstalledEntry> = vec![("upac".to_owned(), "x86_64".to_owned(), Some("v3".to_owned()))];

    let (arch, arch_sub) = find_installed(&installed, "upac").unwrap();

    assert_eq!(arch, "x86_64");
    assert_eq!(arch_sub, Some("v3".to_owned()));
}
