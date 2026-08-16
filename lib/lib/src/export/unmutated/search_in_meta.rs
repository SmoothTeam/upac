// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorKind};
use upac_abi::package::CPackageMeta;
use upac_abi::request::CSearchInMetaRequest;
use upac_abi::response::CSearchInMetaResponse;
use upac_abi::types::{COwned, CVec};

use crate::export::{try_convert_abi, write_error};
use crate::unmutated::search_in_meta::SearchInMetaData;

use upac_types::states::SearchInMetaStateId;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn search_in_meta(
    request_c: CSearchInMetaRequest, response_out: *mut CSearchInMetaResponse, err_out: *mut CError,
) -> i32 {
    let search_in_meta_data = try_convert_abi!(SearchInMetaData::try_from(&request_c), err_out, SearchInMetaStateId);

    let result = catch_unwind(AssertUnwindSafe(|| {
        crate::unmutated::search_in_meta::run(search_in_meta_data)
    }));

    match result {
        Ok(Ok((metas,))) => {
            if !response_out.is_null() {
                unsafe {
                    *response_out = CSearchInMetaResponse::new(CVec::from_owned(
                        metas.into_iter().map(CPackageMeta::from).collect(),
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
            unsafe { write_error(err_out, SearchInMetaStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}
