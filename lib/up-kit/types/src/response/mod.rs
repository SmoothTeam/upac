// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::response::{
    CBootPluginInfo, CDecodeResponse, CSetupBootstrapImportResponse, CSetupBootstrapKernelResponse,
    CSetupPartitionAddResponse,
};
use upac_abi::types::{COwned, CSlice, CVec};
use upac_abi::{CPackageDependency, CPackageMeta, CPackageTrigger};

use upac_macro::{CTryToRust, RustToC};

use crate::booter::BootResourceKind;
use crate::decoder::PackageTrigger;
use crate::error::ErrorKind;
use crate::{PackageDependency, PackageMeta};

pub mod entry;
pub mod unmutated;

#[derive(Debug, Clone, Copy, CTryToRust, RustToC)]
pub struct BootPluginInfo {
    pub abi_version: u32,
    pub kind: BootResourceKind,
}

#[derive(Debug, Clone, CTryToRust, RustToC)]
pub struct SetupPartitionAddResponse {
    pub label: String,
    pub device_path: String,
}

#[derive(Debug, Clone, CTryToRust, RustToC)]
pub struct SetupBootstrapImportResponse {
    pub digest: String,
}

#[derive(Debug, Clone, CTryToRust, RustToC)]
pub struct SetupBootstrapKernelResponse {
    pub digest: String,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct DecodeResponse {
    pub meta: PackageMeta,
    pub dumpfile: String,

    pub dependencies: Vec<PackageDependency>,

    pub triggers: Vec<PackageTrigger>,
}
