// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::CError;
use upac_abi::request::unmutated::CListPrefixRequest;
use upac_abi::response::unmutated::CListPrefixResponse;

use upac_types::error::{Error, try_convert_abi, write_abi_error};
use upac_types::request::unmutated::ListPrefixRequest;
use upac_types::state::unmutated::ListPrefixStateId;

use crate::unmutated::list_prefix::run;

/// # Safety
/// Any borrowed byte-slice fields inside `request_c` must remain valid for the duration of the
/// call. `response_out` and `err_out`, if non-null, must each point to writable storage of the
/// matching type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn list_prefix(
    request_c: CListPrefixRequest, response_out: *mut CListPrefixResponse, err_out: *mut CError,
) -> i32 {
    let list_prefix_request = try_convert_abi!(ListPrefixRequest::try_from(&request_c), err_out, ListPrefixStateId);

    match Error::catch(|| run(list_prefix_request)) {
        Ok(response) => {
            if !response_out.is_null() {
                unsafe { *response_out = response.into() };
            }
            0
        }

        Err(error) => unsafe { write_abi_error(err_out, error) },
    }
}
