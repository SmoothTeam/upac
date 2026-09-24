// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use i18n_embed_fl::fl;

use nix::unistd::Uid;

use upac_abi::SETUP_ABI_VERSION;
use upac_abi::error::CError;
use upac_abi::hook::CancelToken;
use upac_abi::request::bootstrap::CSetupBootstrapRequest;
use upac_abi::request::format::CSetupFormatRequest;
use upac_abi::request::partition::{CSetupPartitionAddRequest, CSetupPartitionTableRequest};
use upac_abi::response::partition::CSetupPartitionAddResponse;

use upac_setup::export::bootstrap::bootstrap_system;
use upac_setup::export::format::format_partition;
use upac_setup::export::partition::{partition_add, partition_table};
use upac_setup::export::{setup_abi_version, setup_cancel};

use crate::locale::LOADER;
use crate::types::errors::AbiMismatch;

pub struct Lib {
    pub partition_table: unsafe extern "C" fn(CSetupPartitionTableRequest, *mut CError) -> i32,
    pub partition_add:
        unsafe extern "C" fn(CSetupPartitionAddRequest, *mut CSetupPartitionAddResponse, *mut CError) -> i32,
    pub format_partition: unsafe extern "C" fn(CSetupFormatRequest, *mut CError) -> i32,
    pub bootstrap_system: unsafe extern "C" fn(CSetupBootstrapRequest, *mut CError) -> i32,

    pub cancel: unsafe extern "C" fn(*mut CancelToken),
    pub version_abi: unsafe extern "C" fn() -> u32,
}

impl Lib {
    pub fn load() -> Result<Self> {
        let lib = Self {
            partition_table,
            partition_add,
            format_partition,
            bootstrap_system,
            cancel: setup_cancel,
            version_abi: setup_abi_version,
        };

        let abi_version = unsafe { (lib.version_abi)() };
        if abi_version != SETUP_ABI_VERSION {
            let err = AbiMismatch {
                got: abi_version,
                expected: SETUP_ABI_VERSION,
            };

            return Err(err.into());
        }

        Ok(lib)
    }

    pub fn require_root(&self) -> Result<()> {
        if !Uid::effective().is_root() {
            anyhow::bail!(fl!(LOADER, "err-requires-root"));
        }

        Ok(())
    }
}
