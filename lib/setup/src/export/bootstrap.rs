// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::CError;
use upac_abi::request::bootstrap::CSetupBootstrapRequest;

use upac_types::error::export_mutated_command;

use crate::commands::bootstrap::run;

/// # Safety
/// Any borrowed byte-slice fields inside `request` must remain valid for the duration of the call.
/// `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bootstrap_system(request: CSetupBootstrapRequest, err_out: *mut CError) -> i32 {
    unsafe {
        export_mutated_command(&request, err_out, |request| {
            run(request).map_err(|(state, error)| (state, error.into()))
        })
    }
}
