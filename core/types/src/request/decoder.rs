// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::request::decoder::CDecodeRequest;
use upac_abi::types::{COwned, CSlice};

use upac_macro::{CTryToRust, RustToC};

use crate::CancelToken;
use crate::error::ErrorKind;

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct DecodeRequest<'data> {
    pub package_path: &'data str,
    pub output_dir: &'data str,
    pub checksum: [u8; 32],
    pub cancel_token: &'data CancelToken,
}
