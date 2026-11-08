// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::package::{CPackageDependency, CPackageMeta, CPackageTrigger};
use upac_abi::response::decoder::CDecodeResponse;
use upac_abi::types::{COwned, CSlice, CVec};

use upac_macro::{CTryToRust, RustToC};

use crate::decoder::PackageTrigger;
use crate::error::ErrorKind;
use crate::package::{PackageDependency, PackageMeta};

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct DecodeResponse {
    pub meta: PackageMeta,
    pub dumpfile: String,
    pub dependencies: Vec<PackageDependency>,
    pub triggers: Vec<PackageTrigger>,
}
