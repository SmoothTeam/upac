// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::request::CRequestBase;
use upac_abi::request::bootstrap::CSetupBootstrapRequest;
use upac_abi::types::{COwned, CSlice};

use upac_macro::{CEnum, CTryToRust, RustToC};

use super::RequestBase;
use crate::error::ErrorKind;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, CEnum)]
pub enum InitramfsGenerator {
    Dracut = 0,
    Mkinitcpio = 1,
}

#[derive(Debug, Clone, RustToC, CTryToRust)]
pub struct SetupBootstrapRequest<'data> {
    pub base: RequestBase<'data>,

    pub disk: Option<&'data str>,
    pub esp_device: Option<&'data str>,
    pub deploy_device: Option<&'data str>,

    pub mount_point: Option<&'data str>,
    pub tmp_path: Option<&'data str>,
    pub source: &'data str,
    pub empty_config: bool,
    pub pinned: bool,

    pub boot_plugin: &'data str,
    pub initramfs_generator: InitramfsGenerator,
}
