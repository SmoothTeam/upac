// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ptr::{eq, null};

use upac_abi::request::decoder::CDecodeRequest;

use upac_types::CancelToken;
use upac_types::error::ErrorKind;
use upac_types::request::decoder::DecodeRequest;

#[test]
fn cancel_token_starts_not_cancelled() {
    let token = CancelToken::new();

    assert!(!token.is_cancelled());
}

#[test]
fn cancel_token_default_starts_not_cancelled() {
    let token = CancelToken::default();

    assert!(!token.is_cancelled());
}

#[test]
fn cancel_token_cancel_is_observed() {
    let token = CancelToken::new();

    token.cancel();

    assert!(token.is_cancelled());
}

#[test]
fn a_static_cancel_token_needs_no_mut() {
    static TOKEN: CancelToken = CancelToken::new();

    TOKEN.cancel();

    assert!(TOKEN.is_cancelled());
}

#[test]
fn a_decode_request_borrows_the_same_cancel_token_back() {
    let token = CancelToken::new();
    let c_request = CDecodeRequest::from(DecodeRequest {
        package_path: "/tmp/package",
        output_dir: "/tmp/output",
        checksum: [7; 32],
        cancel_token: &token,
    });

    let request = DecodeRequest::try_from(&c_request).unwrap();
    token.cancel();

    assert!(eq(request.cancel_token, &token));
    assert!(request.cancel_token.is_cancelled());
    unsafe { c_request.free() };
}

#[test]
fn a_decode_request_without_a_cancel_token_is_rejected() {
    let token = CancelToken::new();
    let mut c_request = CDecodeRequest::from(DecodeRequest {
        package_path: "/tmp/package",
        output_dir: "/tmp/output",
        checksum: [7; 32],
        cancel_token: &token,
    });
    c_request.cancel_token = null();

    assert_eq!(DecodeRequest::try_from(&c_request).err(), Some(ErrorKind::InvalidEntry));
    unsafe { c_request.free() };
}
