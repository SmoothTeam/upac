// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::request::CRequestBase;
use upac_abi::request::partition::{CSetupPartitionAddRequest, CSetupPartitionTableRequest};
use upac_abi::types::{COwned, CSlice};

use upac_macro::{CEnum, CTryToRust, RustToC};

use super::RequestBase;
use crate::error::ErrorKind;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, CEnum)]
pub enum PartitionKind {
    Esp = 0,
    Root = 1,
    Linux = 2,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SetupPartitionTableRequest<'data> {
    pub base: RequestBase<'data>,

    pub device_path: &'data str,
    pub force_wipe: bool,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SetupPartitionAddRequest<'data> {
    pub base: RequestBase<'data>,

    pub device_path: &'data str,
    pub label: &'data str,
    pub size_mib: u64,
    pub kind: PartitionKind,
}
