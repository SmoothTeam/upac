// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::SETUP_ABI_VERSION;
use upac_abi::error::CError;
use upac_abi::request::bootstrap::CSetupBootstrapRequest;
use upac_abi::request::format::CSetupFormatRequest;
use upac_abi::request::partition::{CSetupPartitionAddRequest, CSetupPartitionTableRequest};
use upac_abi::response::partition::CSetupPartitionAddResponse;

use upac_types::error::{export_mutated_command, export_unmutated_command};

use crate::commands::partition::{add, table};
use crate::commands::{bootstrap, format};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn setup_abi_version() -> u32 {
    SETUP_ABI_VERSION
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bootstrap_system(request: CSetupBootstrapRequest, err_out: *mut CError) -> i32 {
    unsafe {
        export_mutated_command(&request, err_out, |request| {
            bootstrap::run(request).map_err(|(state, error)| (state, error.into()))
        })
    }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn format_partition(request: CSetupFormatRequest, err_out: *mut CError) -> i32 {
    unsafe {
        export_mutated_command(&request, err_out, |request| {
            format::run(request).map_err(|(state, error)| (state, error.into()))
        })
    }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn partition_table(request: CSetupPartitionTableRequest, err_out: *mut CError) -> i32 {
    unsafe {
        export_mutated_command(&request, err_out, |request| {
            table::run(request).map_err(|(state, error)| (state, error.into()))
        })
    }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `response_out` and `err_out`, if non-null, must each point to writable storage of the matching
/// type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn partition_add(
    request: CSetupPartitionAddRequest, response_out: *mut CSetupPartitionAddResponse, err_out: *mut CError,
) -> i32 {
    unsafe {
        export_unmutated_command(&request, response_out, err_out, |request| {
            add::run(request).map_err(|(state, error)| (state, error.into()))
        })
    }
}
