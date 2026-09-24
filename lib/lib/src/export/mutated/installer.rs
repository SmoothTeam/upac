// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::CError;
use upac_abi::request::mutated::CInstallRequest;

use upac_types::error::{Error, try_convert_abi, write_abi_error};
use upac_types::request::mutated::InstallRequest;
use upac_types::state::mutated::InstallStateId;

use crate::mutated::installer::run;

/// # Safety
/// Any borrowed byte-slice fields inside `request_c` must remain valid for the duration of the
/// call. `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn install(request_c: CInstallRequest, err_out: *mut CError) -> i32 {
    let install_data = try_convert_abi!(InstallRequest::try_from(&request_c), err_out, InstallStateId);

    match Error::catch(|| run(install_data)) {
        Ok(()) => 0,

        Err(error) => unsafe { write_abi_error(err_out, error) },
    }
}
