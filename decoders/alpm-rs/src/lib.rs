// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::ABI_VERSION;

mod error;
mod extract;
mod verify;

include!(concat!(env!("OUT_DIR"), "/layout.rs"));

/// # Safety
/// Touches no pointers.
#[cfg_attr(feature = "cdylib", unsafe(no_mangle))]
pub unsafe extern "C" fn abi_version() -> u32 {
    ABI_VERSION
}
