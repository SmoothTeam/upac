// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorKind};
use upac_abi::request::CListPrefixRequest;
use upac_abi::response::{CListPrefixResponse, CPrefixEntry};
use upac_abi::types::{COwned, CVec};

use crate::export::{try_convert_abi, write_error};
use crate::unmutated::list_prefix::ListPrefixData;

use upac_types::states::ListPrefixStateId;

/// # Safety
/// Any borrowed byte-slice fields inside `request_c` must remain valid for the duration of the
/// call. `response_out` and `err_out`, if non-null, must each point to writable storage of the
/// matching type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn list_prefix(
    request_c: CListPrefixRequest, response_out: *mut CListPrefixResponse, err_out: *mut CError,
) -> i32 {
    let list_prefix_data = try_convert_abi!(ListPrefixData::try_from(&request_c), err_out, ListPrefixStateId);

    let result = catch_unwind(AssertUnwindSafe(|| {
        crate::unmutated::list_prefix::run(list_prefix_data)
    }));

    match result {
        Ok(Ok((prefixes,))) => {
            if !response_out.is_null() {
                unsafe {
                    *response_out = CListPrefixResponse::new(CVec::from_owned(
                        prefixes.into_iter().map(CPrefixEntry::from).collect(),
                    ));
                }
            }
            0
        }
        Ok(Err((state, error))) => {
            unsafe { write_error(err_out, state, ErrorKind::from(error)) };
            -1
        }
        Err(_) => {
            unsafe { write_error(err_out, ListPrefixStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}
