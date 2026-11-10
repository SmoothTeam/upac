// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::SETUP_ABI_VERSION;
use upac_abi::error::CError;
use upac_abi::request::bootstrap::{
    CSetupBootstrapDeployRequest, CSetupBootstrapImportRequest, CSetupBootstrapKernelRequest,
};
use upac_abi::request::format::CSetupFormatRequest;
use upac_abi::request::partition::{CSetupPartitionAddRequest, CSetupPartitionTableRequest};
use upac_abi::response::bootstrap::{CSetupBootstrapImportResponse, CSetupBootstrapKernelResponse};
use upac_abi::response::partition::CSetupPartitionAddResponse;

use upac_types::error::{export_mutated_command, export_mutated_command_with_response};

use crate::commands::{add, deploy, format, import, init, kernel};

/// # Safety
/// Touches no pointers — `unsafe extern "C"` only to match the ABI calling convention.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setup_abi_version() -> u32 {
    SETUP_ABI_VERSION
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `response_out` and `err_out`, if non-null, must each point to writable storage of the matching
/// type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bootstrap_import(
    request: CSetupBootstrapImportRequest, response_out: *mut CSetupBootstrapImportResponse, err_out: *mut CError,
) -> i32 {
    unsafe { export_mutated_command_with_response(&request, response_out, err_out, import::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `response_out` and `err_out`, if non-null, must each point to writable storage of the matching
/// type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bootstrap_kernel(
    request: CSetupBootstrapKernelRequest, response_out: *mut CSetupBootstrapKernelResponse, err_out: *mut CError,
) -> i32 {
    unsafe { export_mutated_command_with_response(&request, response_out, err_out, kernel::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bootstrap_deploy(request: CSetupBootstrapDeployRequest, err_out: *mut CError) -> i32 {
    unsafe { export_mutated_command(&request, err_out, deploy::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn format_partition(request: CSetupFormatRequest, err_out: *mut CError) -> i32 {
    unsafe { export_mutated_command(&request, err_out, format::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn partition_table(request: CSetupPartitionTableRequest, err_out: *mut CError) -> i32 {
    unsafe { export_mutated_command(&request, err_out, init::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `response_out` and `err_out`, if non-null, must each point to writable storage of the matching
/// type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn partition_add(
    request: CSetupPartitionAddRequest, response_out: *mut CSetupPartitionAddResponse, err_out: *mut CError,
) -> i32 {
    unsafe { export_mutated_command_with_response(&request, response_out, err_out, add::run) }
}
