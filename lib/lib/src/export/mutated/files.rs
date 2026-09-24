// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::CError;
use upac_abi::request::mutated::CFilesRequest;

use upac_types::error::{Error, try_convert_abi, write_abi_error};
use upac_types::request::mutated::FilesRequest;
use upac_types::state::mutated::FilesStateId;

use crate::mutated::files::run;

/// # Safety
/// Any borrowed byte-slice fields inside `request_c` must remain valid for the duration of the
/// call. `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn files(request_c: CFilesRequest, err_out: *mut CError) -> i32 {
    let files_request = try_convert_abi!(FilesRequest::try_from(&request_c), err_out, FilesStateId);

    match Error::catch(|| run(files_request)) {
        Ok(()) => 0,

        Err(error) => unsafe { write_abi_error(err_out, error) },
    }
}
