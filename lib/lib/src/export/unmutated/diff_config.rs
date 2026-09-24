// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::CError;
use upac_abi::request::unmutated::CDiffConfigRequest;
use upac_abi::response::unmutated::CDiffConfigResponse;

use upac_types::error::{Error, try_convert_abi, write_abi_error};
use upac_types::request::unmutated::DiffConfigRequest;
use upac_types::state::unmutated::DiffConfigStateId;

use crate::unmutated::diff_config::run;

/// # Safety
/// Any borrowed byte-slice fields inside `request_c` must remain valid for the duration of the
/// call. `response_out` and `err_out`, if non-null, must each point to writable storage of the
/// matching type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn diff_config(
    request_c: CDiffConfigRequest, response_out: *mut CDiffConfigResponse, err_out: *mut CError,
) -> i32 {
    let diff_config_request = try_convert_abi!(DiffConfigRequest::try_from(&request_c), err_out, DiffConfigStateId);

    match Error::catch(|| run(diff_config_request)) {
        Ok(response) => {
            if !response_out.is_null() {
                unsafe { *response_out = response.into() };
            }
            0
        }

        Err(error) => unsafe { write_abi_error(err_out, error) },
    }
}
