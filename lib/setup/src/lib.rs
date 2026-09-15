// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::SETUP_ABI_VERSION;
use upac_abi::error::{CError, ErrorKind};
use upac_abi::hook::CancelToken;
use upac_abi::request::{CSetupExistingRequest, CSetupWholeDiskRequest};

use upac_types::error::{try_convert_abi, write_error};
use upac_types::states::SetupStateId;

use self::stages::{SetupExistingData, SetupWholeDiskData, run_existing, run_whole_disk};

mod archive;
mod error;
mod format;
mod stages;
mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}
mod partition;
mod target;

/// # Safety
/// Touches no pointers — `unsafe extern "C"` only to match the ABI calling convention.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setup_abi_version() -> u32 {
    SETUP_ABI_VERSION
}

/// # Safety
/// `token`, if non-null, must point to a valid, initialized `CancelToken` for the duration of the
/// call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cancel(token: *mut CancelToken) {
    if token.is_null() {
        return;
    }
    unsafe { (*token).cancel() };
}

/// # Safety
/// Any borrowed byte-slice fields inside `request_c` must remain valid for the duration of the
/// call. `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setup_existing(request_c: CSetupExistingRequest, err_out: *mut CError) -> i32 {
    let data = try_convert_abi!(SetupExistingData::try_from(&request_c), err_out, SetupStateId);

    let result = catch_unwind(AssertUnwindSafe(|| run_existing(data)));

    match result {
        Ok(Ok(())) => 0,

        Ok(Err((state, error))) => {
            unsafe { write_error(err_out, state, ErrorKind::from(error)) };
            -1
        }

        Err(_) => {
            unsafe { write_error(err_out, SetupStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request_c` must remain valid for the duration of the
/// call. `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setup_whole_disk(request_c: CSetupWholeDiskRequest, err_out: *mut CError) -> i32 {
    let data = try_convert_abi!(SetupWholeDiskData::try_from(&request_c), err_out, SetupStateId);

    let result = catch_unwind(AssertUnwindSafe(|| run_whole_disk(data)));

    match result {
        Ok(Ok(())) => 0,

        Ok(Err((state, error))) => {
            unsafe { write_error(err_out, state, ErrorKind::from(error)) };
            -1
        }

        Err(_) => {
            unsafe { write_error(err_out, SetupStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}
