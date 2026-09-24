// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::CError;
use upac_abi::request::unmutated::CSearchFilesRequest;
use upac_abi::response::unmutated::CSearchFilesResponse;

use upac_types::error::{Error, try_convert_abi, write_abi_error};
use upac_types::request::unmutated::SearchFilesRequest;
use upac_types::state::unmutated::SearchFilesStateId;

use crate::unmutated::search_files::run;

/// # Safety
/// Any borrowed byte-slice fields inside `request_c` must remain valid for the duration of the
/// call. `response_out` and `err_out`, if non-null, must each point to writable storage of the
/// matching type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn search_files(
    request_c: CSearchFilesRequest, response_out: *mut CSearchFilesResponse, err_out: *mut CError,
) -> i32 {
    let search_files_request = try_convert_abi!(SearchFilesRequest::try_from(&request_c), err_out, SearchFilesStateId);

    match Error::catch(|| run(search_files_request)) {
        Ok(response) => {
            if !response_out.is_null() {
                unsafe { *response_out = response.into() };
            }
            0
        }

        Err(error) => unsafe { write_abi_error(err_out, error) },
    }
}
