// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::SETUP_ABI_VERSION;

pub mod bootstrap;
pub mod format;
pub mod partition;

/// # Safety
/// Touches no pointers — `unsafe extern "C"` only to match the ABI calling convention.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setup_abi_version() -> u32 {
    SETUP_ABI_VERSION
}
