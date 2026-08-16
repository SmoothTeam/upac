// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::mem::size_of;
use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorKind};
use upac_abi::request::CDiffPrefixRequest;
use upac_abi::response::{CDiffPrefixFileEntry, CDiffPrefixResponse};
use upac_abi::types::{COwned, CVec};

use crate::export::{try_convert_abi, write_error};
use crate::unmutated::diff_prefix::DiffPrefixData;

use upac_types::states::DiffPrefixStateId;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn diff_prefix(
    request_c: CDiffPrefixRequest, response_out: *mut CDiffPrefixResponse, err_out: *mut CError,
) -> i32 {
    let diff_prefix_data = try_convert_abi!(DiffPrefixData::try_from(&request_c), err_out, DiffPrefixStateId);

    let result = catch_unwind(AssertUnwindSafe(|| {
        crate::unmutated::diff_prefix::run(diff_prefix_data)
    }));

    match result {
        Ok(Ok((files,))) => {
            if !response_out.is_null() {
                unsafe {
                    *response_out = CDiffPrefixResponse {
                        struct_size: size_of::<CDiffPrefixResponse>(),
                        files: CVec::from_owned(files.into_iter().map(CDiffPrefixFileEntry::from).collect()),
                    };
                }
            }
            0
        }
        Ok(Err((state, error))) => {
            unsafe { write_error(err_out, state, ErrorKind::from(error)) };
            -1
        }
        Err(_) => {
            unsafe { write_error(err_out, DiffPrefixStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}
