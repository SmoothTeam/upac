// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::response::bootstrap::{CSetupBootstrapImportResponse, CSetupBootstrapKernelResponse};
use upac_abi::types::{COwned, CSlice};

use upac_macro::{CTryToRust, RustToC};

use crate::error::ErrorKind;

#[derive(Debug, Clone, CTryToRust, RustToC)]
pub struct SetupBootstrapImportResponse {
    pub digest: String,
}

#[derive(Debug, Clone, CTryToRust, RustToC)]
pub struct SetupBootstrapKernelResponse {
    pub digest: String,
}
