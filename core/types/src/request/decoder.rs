// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::CancelToken;
use upac_abi::request::decoder::CDecodeRequest;
use upac_abi::types::{COwned, CSlice};

use upac_macro::RustToC;

#[derive(Debug, Clone, RustToC)]
pub struct DecodeRequest {
    pub package_path: String,
    pub output_dir: String,
    pub checksum: [u8; 32],
    pub cancel_token: *mut CancelToken,
}
