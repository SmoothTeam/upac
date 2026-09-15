// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::mem::MaybeUninit;

use anyhow::Result;

use i18n_embed_fl::fl;

use nix::unistd::Uid;

use upac_abi::SETUP_ABI_VERSION;
use upac_abi::error::CError;
use upac_abi::hook::CancelToken;
use upac_abi::request::{CSetupExistingRequest, CSetupWholeDiskRequest};

use upac_setup::{setup_abi_version, setup_cancel, setup_existing, setup_whole_disk};

use crate::locale::LOADER;
use crate::types::errors::{AbiMismatch, LibError};

pub struct Lib {
    pub setup_existing: unsafe extern "C" fn(CSetupExistingRequest, *mut CError) -> i32,
    pub setup_whole_disk: unsafe extern "C" fn(CSetupWholeDiskRequest, *mut CError) -> i32,

    pub cancel: unsafe extern "C" fn(*mut CancelToken),
    pub version_abi: unsafe extern "C" fn() -> u32,
}

impl Lib {
    pub fn load() -> Result<Self> {
        let lib = Self {
            setup_existing,
            setup_whole_disk,
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

pub fn invoke(call: impl FnOnce(*mut CError) -> i32) -> Result<()> {
    let mut error = MaybeUninit::uninit();

    let code = call(error.as_mut_ptr());

    unsafe { LibError::check(code, error.as_ptr())? };

    Ok(())
}
