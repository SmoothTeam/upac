// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ptr::null;

use upac_abi::error::{CError, ErrorDomain, ErrorKind};

use super::LibError;

#[test]
fn check_returns_ok_on_a_zero_code() {
    assert!(unsafe { LibError::check(0, null()) }.is_ok());
}

#[test]
fn check_returns_the_error_on_a_nonzero_code() {
    let error = CError {
        domain: ErrorDomain::Install,
        state: 0,
        error: ErrorKind::NotFound,
    };

    let lib_error = unsafe { LibError::check(1, &error) }.unwrap_err();

    assert_eq!(lib_error.error.domain, ErrorDomain::Install);
    assert_eq!(lib_error.error.state, 0);
    assert_eq!(lib_error.error.error, ErrorKind::NotFound);
}
