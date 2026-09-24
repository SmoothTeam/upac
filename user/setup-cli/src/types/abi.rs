// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::mem::MaybeUninit;

use anyhow::Result;

use upac_abi::error::CError;
use upac_abi::types::CValidatable;

use upac_types::error::Error as AbiError;

use crate::types::errors::{InvalidResponse, LibError};

pub fn invoke(call: impl FnOnce(*mut CError) -> i32) -> Result<()> {
    let mut error = CError::default();

    let code = call(&mut error);

    AbiError::check(code, &error).map_err(LibError)?;

    Ok(())
}

pub fn invoke_with_response<R: CValidatable>(call: impl FnOnce(*mut R, *mut CError) -> i32) -> Result<R> {
    let mut response = MaybeUninit::zeroed();

    let mut error = CError::default();

    let code = call(response.as_mut_ptr(), &mut error);

    AbiError::check(code, &error).map_err(LibError)?;

    let response = unsafe { response.assume_init() };

    unsafe { response.validate() }.map_err(|error| InvalidResponse { error })?;

    Ok(response)
}
