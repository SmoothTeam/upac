// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::CError;
use upac_abi::request::unmutated::{
    CDiffConfigRequest, CDiffPackagesRequest, CDiffPrefixRequest, CDiffRequest, CListConfigRequest,
    CListHistoryRequest, CListPackagesRequest, CListPrefixRequest, CSearchFilesRequest, CSearchInMetaRequest,
    CSearchInPackageFilesRequest, CSearchMetaRequest,
};
use upac_abi::response::unmutated::{
    CDiffConfigResponse, CDiffPackagesResponse, CDiffPrefixResponse, CDiffResponse, CListConfigResponse,
    CListHistoryResponse, CListPackagesResponse, CListPrefixResponse, CSearchFilesResponse, CSearchInMetaResponse,
    CSearchInPackageFilesResponse, CSearchMetaResponse,
};

use upac_types::error::export_unmutated_command;

use crate::unmutated::{
    diff, diff_config, diff_packages, diff_prefix, list_config, list_history, list_packages, list_prefix, search_files,
    search_in_meta, search_in_package_files, search_meta,
};

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `response_out` and `err_out`, if non-null, must each point to writable storage of the matching
/// type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn list_packages(
    request: CListPackagesRequest, response_out: *mut CListPackagesResponse, err_out: *mut CError,
) -> i32 {
    unsafe { export_unmutated_command(&request, response_out, err_out, list_packages::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `response_out` and `err_out`, if non-null, must each point to writable storage of the matching
/// type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn list_config(
    request: CListConfigRequest, response_out: *mut CListConfigResponse, err_out: *mut CError,
) -> i32 {
    unsafe { export_unmutated_command(&request, response_out, err_out, list_config::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `response_out` and `err_out`, if non-null, must each point to writable storage of the matching
/// type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn list_prefix(
    request: CListPrefixRequest, response_out: *mut CListPrefixResponse, err_out: *mut CError,
) -> i32 {
    unsafe { export_unmutated_command(&request, response_out, err_out, list_prefix::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `response_out` and `err_out`, if non-null, must each point to writable storage of the matching
/// type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn list_history(
    request: CListHistoryRequest, response_out: *mut CListHistoryResponse, err_out: *mut CError,
) -> i32 {
    unsafe { export_unmutated_command(&request, response_out, err_out, list_history::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `response_out` and `err_out`, if non-null, must each point to writable storage of the matching
/// type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn diff(request: CDiffRequest, response_out: *mut CDiffResponse, err_out: *mut CError) -> i32 {
    unsafe { export_unmutated_command(&request, response_out, err_out, diff::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `response_out` and `err_out`, if non-null, must each point to writable storage of the matching
/// type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn diff_prefix(
    request: CDiffPrefixRequest, response_out: *mut CDiffPrefixResponse, err_out: *mut CError,
) -> i32 {
    unsafe { export_unmutated_command(&request, response_out, err_out, diff_prefix::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `response_out` and `err_out`, if non-null, must each point to writable storage of the matching
/// type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn diff_config(
    request: CDiffConfigRequest, response_out: *mut CDiffConfigResponse, err_out: *mut CError,
) -> i32 {
    unsafe { export_unmutated_command(&request, response_out, err_out, diff_config::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `response_out` and `err_out`, if non-null, must each point to writable storage of the matching
/// type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn diff_packages(
    request: CDiffPackagesRequest, response_out: *mut CDiffPackagesResponse, err_out: *mut CError,
) -> i32 {
    unsafe { export_unmutated_command(&request, response_out, err_out, diff_packages::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `response_out` and `err_out`, if non-null, must each point to writable storage of the matching
/// type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn search_meta(
    request: CSearchMetaRequest, response_out: *mut CSearchMetaResponse, err_out: *mut CError,
) -> i32 {
    unsafe { export_unmutated_command(&request, response_out, err_out, search_meta::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `response_out` and `err_out`, if non-null, must each point to writable storage of the matching
/// type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn search_files(
    request: CSearchFilesRequest, response_out: *mut CSearchFilesResponse, err_out: *mut CError,
) -> i32 {
    unsafe { export_unmutated_command(&request, response_out, err_out, search_files::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `response_out` and `err_out`, if non-null, must each point to writable storage of the matching
/// type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn search_in_meta(
    request: CSearchInMetaRequest, response_out: *mut CSearchInMetaResponse, err_out: *mut CError,
) -> i32 {
    unsafe { export_unmutated_command(&request, response_out, err_out, search_in_meta::run) }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `response_out` and `err_out`, if non-null, must each point to writable storage of the matching
/// type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn search_in_package_files(
    request: CSearchInPackageFilesRequest, response_out: *mut CSearchInPackageFilesResponse, err_out: *mut CError,
) -> i32 {
    unsafe { export_unmutated_command(&request, response_out, err_out, search_in_package_files::run) }
}
