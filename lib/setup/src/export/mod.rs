// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::SETUP_ABI_VERSION;
use upac_abi::hook::CancelToken;

pub mod bootstrap;
pub mod format;
pub mod partition;

/// # Safety
/// Touches no pointers — `unsafe extern "C"` only to match the ABI calling convention.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setup_abi_version() -> u32 {
    SETUP_ABI_VERSION
}

/// # Safety
/// `token`, if non-null, must point to a valid, initialized `CancelToken` for the duration of the
/// call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setup_cancel(token: *mut CancelToken) {
    if token.is_null() {
        return;
    }
    unsafe { (*token).cancel() };
}
