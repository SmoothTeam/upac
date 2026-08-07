// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::ptr::{null, null_mut};

use upac_abi::error::ErrorKind;
use upac_abi::types::{CBorrowed, COwned, CSlice, CVec};

#[test]
fn cslice_from_slice_none_is_null() {
    let slice = CSlice::from_slice(None);

    assert!(slice.ptr.is_null());
    assert_eq!(slice.len, 0);
}

#[test]
fn cslice_owned_round_trips_through_as_str() {
    let owned = CSlice::from_owned(b"hello".to_vec());

    assert_eq!(unsafe { owned.as_str() }, Ok("hello"));
    assert_eq!(unsafe { owned.into_owned() }, b"hello".to_vec());
}

#[test]
fn cslice_borrowed_round_trips_through_as_slice() {
    let bytes = b"borrowed".to_vec();
    let borrowed = CSlice::from_borrowed(bytes.as_slice());

    assert_eq!(unsafe { borrowed.as_slice() }, bytes.as_slice());
}

#[test]
fn cslice_null_validate_fails() {
    let slice = CSlice { ptr: null(), len: 0 };

    assert_eq!(unsafe { slice.validate() }, Err(ErrorKind::InvalidEntry));
}

#[test]
fn cvec_owned_round_trips() {
    let vec: CVec<u32> = CVec::from_owned(vec![1, 2, 3]);

    assert_eq!(unsafe { vec.as_slice() }, &[1, 2, 3]);
    assert_eq!(unsafe { vec.into_owned() }, vec![1, 2, 3]);
}

#[test]
fn cvec_empty_owned_is_null() {
    let vec: CVec<u32> = CVec::from_owned(Vec::new());

    assert!(vec.ptr.is_null());
    assert_eq!(vec.len, 0);
    assert!(unsafe { vec.validate() }.is_ok());
}

#[test]
fn cvec_null_with_nonzero_len_fails_validate() {
    let vec: CVec<u32> = CVec {
        ptr: null_mut(),
        len: 3,
    };

    assert_eq!(unsafe { vec.validate() }, Err(ErrorKind::InvalidEntry));
}
