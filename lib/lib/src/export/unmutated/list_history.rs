// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorKind};
use upac_abi::request::CListHistoryRequest;
use upac_abi::response::{CHistoryEntry, CListHistoryResponse};
use upac_abi::types::{COwned, CVec};

use crate::export::{try_convert_abi, write_error};
use crate::unmutated::list_history::ListHistoryData;

use upac_types::states::ListHistoryStateId;

/// # Safety
/// Any borrowed byte-slice fields inside `request_c` must remain valid for the duration of the
/// call. `response_out` and `err_out`, if non-null, must each point to writable storage of the
/// matching type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn list_history(
    request_c: CListHistoryRequest, response_out: *mut CListHistoryResponse, err_out: *mut CError,
) -> i32 {
    let list_history_data = try_convert_abi!(ListHistoryData::try_from(&request_c), err_out, ListHistoryStateId);

    let result = catch_unwind(AssertUnwindSafe(|| {
        crate::unmutated::list_history::run(list_history_data)
    }));

    match result {
        Ok(Ok((history,))) => {
            if !response_out.is_null() {
                unsafe {
                    *response_out = CListHistoryResponse::new(CVec::from_owned(
                        history.into_iter().map(CHistoryEntry::from).collect(),
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
            unsafe { write_error(err_out, ListHistoryStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}
