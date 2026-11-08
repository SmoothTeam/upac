// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::os::raw::c_void;

use upac_abi::package::CPackageChecksum;
use upac_abi::plugin::{CreateFileFn, FinishFileFn};
use upac_abi::request::decoder::CDecodeRequest;
use upac_abi::types::{COwned, CSlice};

use upac_macro::{CTryToRust, RustToC};

use crate::CancelToken;
use crate::error::ErrorKind;
use crate::package::PackageChecksum;

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct DecodeRequest<'data> {
    pub package_path: &'data str,
    pub checksum: PackageChecksum,
    pub create_file: CreateFileFn,
    pub finish_file: FinishFileFn,
    pub store_ctx: *mut c_void,
    pub cancel_token: &'data CancelToken,
}
