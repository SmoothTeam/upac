// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::CProgressEvent;

use upac_macro::RustToC;

#[derive(Debug, Clone, PartialEq, Eq, RustToC)]
pub struct ProgressEvent<'event> {
    pub stage: u32,
    pub subject: Option<&'event str>,
    pub current: u64,
    pub total: u64,
}
