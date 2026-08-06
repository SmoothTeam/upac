// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::mem::size_of;
use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorKind};
use upac_abi::request::CListCommitRequest;
use upac_abi::response::{CCommitEntry, CListCommitResponse};
use upac_abi::types::{COwned, CVec};

use crate::export::{try_convert_abi, write_error};
use crate::types::states::ListCommitStateId;
use crate::unmutated::list_commit::ListCommitData;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn list_commit(
    request_c: CListCommitRequest, response_out: *mut CListCommitResponse, err_out: *mut CError,
) -> i32 {
    let list_commit_data = try_convert_abi!(ListCommitData::try_from(&request_c), err_out, ListCommitStateId);

    let result = catch_unwind(AssertUnwindSafe(|| {
        crate::unmutated::list_commit::run(list_commit_data)
    }));

    match result {
        Ok(Ok((commits,))) => {
            if !response_out.is_null() {
                unsafe {
                    *response_out = CListCommitResponse {
                        struct_size: size_of::<CListCommitResponse>(),
                        commits: CVec::from_owned(commits.into_iter().map(CCommitEntry::from).collect()),
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
            unsafe { write_error(err_out, ListCommitStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}
