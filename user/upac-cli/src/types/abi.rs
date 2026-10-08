// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::mem::MaybeUninit;

use anyhow::Result;

use upac_abi::error::CError;
use upac_abi::types::CValidatable;

use upac_types::diff::DiffFileSource;
use upac_types::error::Error as AbiError;

use crate::types::errors::{InvalidResponse, LibError};

#[cfg(test)]
#[path = "../../tests/inline/abi.rs"]
mod tests;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum FileScope {
    Usr,
    Config,
}

impl From<FileScope> for DiffFileSource {
    fn from(value: FileScope) -> Self {
        match value {
            FileScope::Usr => DiffFileSource::Prefix,
            FileScope::Config => DiffFileSource::Config,
        }
    }
}

pub fn invoke(call: impl FnOnce(*mut CError) -> i32) -> Result<()> {
    let mut error = CError::default();

    let code = call(&mut error);

    let checked = AbiError::check(code, &error);
    unsafe { error.free() };
    checked.map_err(LibError)?;

    Ok(())
}

pub fn invoke_with_response<Response: CValidatable>(
    call: impl FnOnce(*mut Response, *mut CError) -> i32,
) -> Result<Response> {
    let mut response = MaybeUninit::zeroed();

    let mut error = CError::default();

    let code = call(response.as_mut_ptr(), &mut error);

    let checked = AbiError::check(code, &error);
    unsafe { error.free() };
    checked.map_err(LibError)?;

    let response = unsafe { response.assume_init() };

    unsafe { response.validate() }.map_err(|error| InvalidResponse { error: error.into() })?;

    Ok(response)
}
