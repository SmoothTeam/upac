// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::str::from_utf8;

use upac_abi::BOOT_ABI_VERSION;
use upac_abi::boot::{Booter, CBootPluginRequest};
use upac_abi::error::ErrorKind;
use upac_abi::types::CBorrowed;

use crate::backend::Uki;
use crate::error::UkiError;

mod backend;
mod error;

include!(concat!(env!("OUT_DIR"), "/layout.rs"));

#[unsafe(no_mangle)]
pub unsafe extern "C" fn abi_version() -> u32 {
    BOOT_ABI_VERSION
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe() -> i32 {
    i32::from(Uki::probes())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_one_shot(request: *const CBootPluginRequest, err_out: *mut ErrorKind) -> i32 {
    if request.is_null() {
        write_error(err_out, UkiError::InvalidRequest);
        return -1;
    }

    let result = entry_name_from_request(unsafe { &*request })
        .and_then(|entry_name| Uki::new().and_then(|mut uki| uki.set_one_shot(&entry_name)));

    match result {
        Ok(()) => 0,
        Err(error) => {
            write_error(err_out, error);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn confirm_boot(request: *const CBootPluginRequest, err_out: *mut ErrorKind) -> i32 {
    if request.is_null() {
        write_error(err_out, UkiError::InvalidRequest);
        return -1;
    }

    let result = entry_name_from_request(unsafe { &*request })
        .and_then(|entry_name| Uki::new().and_then(|mut uki| uki.confirm_boot(&entry_name)));

    match result {
        Ok(()) => 0,
        Err(error) => {
            write_error(err_out, error);
            -1
        }
    }
}

fn entry_name_from_request(request: &CBootPluginRequest) -> Result<String, UkiError> {
    let bytes = unsafe { request.entry_name.as_borrowed() };

    from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|_| UkiError::InvalidRequest)
}

fn write_error(err_out: *mut ErrorKind, error: UkiError) {
    if !err_out.is_null() {
        unsafe { *err_out = error.into() };
    }
}
