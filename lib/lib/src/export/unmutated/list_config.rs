// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorKind};
use upac_abi::request::CListConfigRequest;
use upac_abi::response::{CConfigCommitEntry, CListConfigResponse};
use upac_abi::types::{COwned, CVec};

use crate::export::{try_convert_abi, write_error};
use crate::unmutated::list_config::ListConfigData;

use upac_types::states::ListConfigStateId;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn list_config(
    request_c: CListConfigRequest, response_out: *mut CListConfigResponse, err_out: *mut CError,
) -> i32 {
    let list_config_data = try_convert_abi!(ListConfigData::try_from(&request_c), err_out, ListConfigStateId);

    let result = catch_unwind(AssertUnwindSafe(|| {
        crate::unmutated::list_config::run(list_config_data)
    }));

    match result {
        Ok(Ok((commits,))) => {
            if !response_out.is_null() {
                unsafe {
                    *response_out = CListConfigResponse::new(CVec::from_owned(
                        commits.into_iter().map(CConfigCommitEntry::from).collect(),
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
            unsafe { write_error(err_out, ListConfigStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}
