// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::mem::size_of;
use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorKind};
use upac_abi::request::CSearchInPackageFilesRequest;
use upac_abi::response::{CSearchFileEntry, CSearchInPackageFilesResponse};
use upac_abi::types::{COwned, CVec};

use crate::export::{try_convert_abi, write_error};
use crate::unmutated::search_in_package_files::SearchInPackageFilesData;

use upac_types::states::SearchInPackageFilesStateId;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn search_in_package_files(
    request_c: CSearchInPackageFilesRequest, response_out: *mut CSearchInPackageFilesResponse, err_out: *mut CError,
) -> i32 {
    let search_in_package_files_data = try_convert_abi!(
        SearchInPackageFilesData::try_from(&request_c),
        err_out,
        SearchInPackageFilesStateId
    );

    let result = catch_unwind(AssertUnwindSafe(|| {
        crate::unmutated::search_in_package_files::run(search_in_package_files_data)
    }));

    match result {
        Ok(Ok((files,))) => {
            if !response_out.is_null() {
                unsafe {
                    *response_out = CSearchInPackageFilesResponse {
                        struct_size: size_of::<CSearchInPackageFilesResponse>(),
                        files: CVec::from_owned(files.into_iter().map(CSearchFileEntry::from).collect()),
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
            unsafe { write_error(err_out, SearchInPackageFilesStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}
