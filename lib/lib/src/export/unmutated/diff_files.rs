// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::mem::size_of;
use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorKind};
use upac_abi::request::CDiffFilesRequest;
use upac_abi::response::{CDiffFileEntry, CDiffFilesResponse};
use upac_abi::types::{COwned, CVec};

use crate::export::{try_convert_abi, write_error};
use crate::types::states::DiffFilesStateId;
use crate::unmutated::diff_files::DiffFilesData;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn diff_files(
    request_c: CDiffFilesRequest, response_out: *mut CDiffFilesResponse, err_out: *mut CError,
) -> i32 {
    let diff_files_data = try_convert_abi!(DiffFilesData::try_from(&request_c), err_out, DiffFilesStateId);

    let result = catch_unwind(AssertUnwindSafe(|| crate::unmutated::diff_files::run(diff_files_data)));

    match result {
        Ok(Ok((files,))) => {
            if !response_out.is_null() {
                unsafe {
                    *response_out = CDiffFilesResponse {
                        struct_size: size_of::<CDiffFilesResponse>(),
                        files: CVec::from_owned(files.into_iter().map(CDiffFileEntry::from).collect()),
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
            unsafe { write_error(err_out, DiffFilesStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}
