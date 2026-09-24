// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::CError;
use upac_abi::request::unmutated::CListHistoryRequest;
use upac_abi::response::unmutated::CListHistoryResponse;

use upac_types::error::{Error, try_convert_abi, write_abi_error};
use upac_types::request::unmutated::ListHistoryRequest;
use upac_types::state::unmutated::ListHistoryStateId;

use crate::unmutated::list_history::run;

/// # Safety
/// Any borrowed byte-slice fields inside `request_c` must remain valid for the duration of the
/// call. `response_out` and `err_out`, if non-null, must each point to writable storage of the
/// matching type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn list_history(
    request_c: CListHistoryRequest, response_out: *mut CListHistoryResponse, err_out: *mut CError,
) -> i32 {
    let list_history_request = try_convert_abi!(ListHistoryRequest::try_from(&request_c), err_out, ListHistoryStateId);

    match Error::catch(|| run(list_history_request)) {
        Ok(response) => {
            if !response_out.is_null() {
                unsafe { *response_out = response.into() };
            }
            0
        }

        Err(error) => unsafe { write_abi_error(err_out, error) },
    }
}
