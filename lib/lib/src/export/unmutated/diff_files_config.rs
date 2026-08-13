// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::mem::size_of;
use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorKind};
use upac_abi::request::CDiffFilesConfigRequest;
use upac_abi::response::{CDiffConfigFileEntry, CDiffFilesConfigResponse};
use upac_abi::types::{COwned, CVec};

use crate::export::{try_convert_abi, write_error};
use crate::types::states::DiffFilesConfigStateId;
use crate::unmutated::diff_files_config::DiffFilesConfigData;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn diff_files_config(
    request_c: CDiffFilesConfigRequest, response_out: *mut CDiffFilesConfigResponse, err_out: *mut CError,
) -> i32 {
    let diff_files_config_data = try_convert_abi!(
        DiffFilesConfigData::try_from(&request_c),
        err_out,
        DiffFilesConfigStateId
    );

    let result = catch_unwind(AssertUnwindSafe(|| {
        crate::unmutated::diff_files_config::run(diff_files_config_data)
    }));

    match result {
        Ok(Ok((files,))) => {
            if !response_out.is_null() {
                unsafe {
                    *response_out = CDiffFilesConfigResponse {
                        struct_size: size_of::<CDiffFilesConfigResponse>(),
                        files: CVec::from_owned(files.into_iter().map(CDiffConfigFileEntry::from).collect()),
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
            unsafe { write_error(err_out, DiffFilesConfigStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}
