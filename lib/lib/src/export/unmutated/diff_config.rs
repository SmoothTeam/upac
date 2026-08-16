// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::mem::size_of;
use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorKind};
use upac_abi::request::CDiffConfigRequest;
use upac_abi::response::{CDiffConfigFileEntry, CDiffConfigResponse};
use upac_abi::types::{COwned, CVec};

use upac_types::states::DiffConfigStateId;

use crate::export::{try_convert_abi, write_error};
use crate::unmutated::diff_config::DiffConfigData;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn diff_config(
    request_c: CDiffConfigRequest, response_out: *mut CDiffConfigResponse, err_out: *mut CError,
) -> i32 {
    let diff_config_data = try_convert_abi!(DiffConfigData::try_from(&request_c), err_out, DiffConfigStateId);

    let result = catch_unwind(AssertUnwindSafe(|| {
        crate::unmutated::diff_config::run(diff_config_data)
    }));

    match result {
        Ok(Ok((files,))) => {
            if !response_out.is_null() {
                unsafe {
                    *response_out = CDiffConfigResponse {
                        struct_size: size_of::<CDiffConfigResponse>(),
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
            unsafe { write_error(err_out, DiffConfigStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}
