// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::{CError, ErrorKind};
use upac_abi::request::CUpdateRequest;

use crate::export::{try_convert_abi, write_error};
use crate::mutated::update::UpdateData;
use crate::types::states::UpdateStateId;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn update(request_c: CUpdateRequest, err_out: *mut CError) -> i32 {
    let update_data = try_convert_abi!(UpdateData::try_from(&request_c), err_out, UpdateStateId);

    let result = catch_unwind(AssertUnwindSafe(|| crate::mutated::update::run(update_data)));

    match result {
        Ok(Ok(())) => 0,
        Ok(Err((state, error))) => {
            unsafe { write_error(err_out, state, ErrorKind::from(error)) };
            -1
        }
        Err(_) => {
            unsafe { write_error(err_out, UpdateStateId::Setup, ErrorKind::Unexpected) };
            -1
        }
    }
}
