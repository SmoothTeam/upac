// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorKind};
use upac_abi::request::mutated::CMimeSyncRequest;

use upac_types::error::{try_convert_abi, write_error};
use upac_types::request::mutated::MimeSyncRequest;
use upac_types::state::mutated::MimeStateId;

use crate::mutated::mime::run;

/// # Safety
/// Any borrowed byte-slice fields inside `request_c` must remain valid for the duration of the
/// call. `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mime(request_c: CMimeSyncRequest, err_out: *mut CError) -> i32 {
    let mime_request = try_convert_abi!(MimeSyncRequest::try_from(&request_c), err_out, MimeStateId);

    let result = catch_unwind(AssertUnwindSafe(|| run(mime_request)));

    match result {
        Ok(Ok(())) => 0,

        Ok(Err((state, error))) => {
            unsafe { write_error(err_out, state, ErrorKind::from(error)) };
            -1
        }

        Err(_) => {
            unsafe { write_error(err_out, MimeStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}
